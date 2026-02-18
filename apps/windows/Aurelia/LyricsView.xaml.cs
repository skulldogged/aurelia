using aurelia_lyrics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Animation;
using Windows.Foundation;
using Windows.UI;
using Windows.UI.Text;

namespace Aurelia {
  public sealed partial class LyricsView {
    private ParsedLyrics? _lyrics;
    private int _lastActiveIndex = -1;
    private readonly List<LineDisplayItem> _lineItems = [];

    // Static colors for brush construction
    private static readonly Color ColorWordPast = Color.FromArgb(0xF2, 0xFF, 0xFF, 0xFF);
    private static readonly Color ColorWordFuture = Color.FromArgb(0x55, 0xFF, 0xFF, 0xFF);

    // Shared static brushes for words that never need a sweep (past/future)
    private static readonly SolidColorBrush BrushWordPast = new(ColorWordPast);
    private static readonly SolidColorBrush BrushWordFuture = new(ColorWordFuture);

    // Each word-synced line gets a per-line LinearGradientBrush for the L-R sweep.
    // Only the two middle GradientStop.Offset values are mutated per tick (zero allocations).
    private class LineDisplayItem {
      public long TimeMs { get; init; }
      public FrameworkElement Container { get; init; } = null!;
      public List<(Run Run, long TimeMs)>? Words { get; init; }
      public int CurrentWordIdx = -1;

      // Only non-null for word-synced lines
      public LinearGradientBrush? SweepBrush { get; init; }
      public GradientStop? SweepBrightStop { get; init; }  // offset = t (bright side)
      public GradientStop? SweepDimStop { get; init; }  // offset = t (dim side, sharp edge)
    }

    public LyricsView() {
      InitializeComponent();
      LyricsScroller.SizeChanged += LyricsScroller_SizeChanged;
      Loaded += (s, e) => UpdateFadeColors();
      ActualThemeChanged += (s, e) => UpdateFadeColors();
    }

    private void LyricsScroller_SizeChanged(object sender, SizeChangedEventArgs e) {
      double half = LyricsScroller.ActualHeight / 2;
      if (half > 16)
        LyricsPanel.Padding = new Thickness(24, half, 24, half);
    }

    private void UpdateFadeColors() {
      // Read the page background color. In WinUI 3 the indexer resolves theme resources;
      // TryGetValue doesn't. Fall back to known Windows 11 defaults if lookup fails.
      Color bg;
      try {
        object? res = Application.Current.Resources["ApplicationPageBackgroundThemeBrush"];
        bg = res is SolidColorBrush { Color.A: > 0 } scb
          ? scb.Color
          : ThemeFallbackColor();
      } catch {
        bg = ThemeFallbackColor();
      }

      Color transparent = Color.FromArgb(0, bg.R, bg.G, bg.B);

      TopFade.Background = new LinearGradientBrush {
        StartPoint = new Point(0, 0),
        EndPoint = new Point(0, 1),
        GradientStops = {
          new GradientStop { Color = bg,          Offset = 0 },
          new GradientStop { Color = transparent,  Offset = 1 }
        }
      };

      BottomFade.Background = new LinearGradientBrush {
        StartPoint = new Point(0, 0),
        EndPoint = new Point(0, 1),
        GradientStops = {
          new GradientStop { Color = transparent,  Offset = 0 },
          new GradientStop { Color = bg,           Offset = 1 }
        }
      };
    }

    private Color ThemeFallbackColor() {
      return ActualTheme == ElementTheme.Light
        ? Color.FromArgb(0xFF, 0xF3, 0xF3, 0xF3)   // Windows 11 light default
        : Color.FromArgb(0xFF, 0x1E, 0x1E, 0x1E);   // Windows 11 dark default
    }

    // ── Public API ────────────────────────────────────────────────────────────

    public ParsedLyrics? Lyrics {
      get => _lyrics;
      set { _lyrics = value; BuildLyricsDisplay(); }
    }

    public void SetLoading(bool loading) {
      LoadingRing.Visibility = loading ? Visibility.Visible : Visibility.Collapsed;
      NoLyricsText.Visibility = Visibility.Collapsed;
      if (!loading) {
        return;
      }

      LyricsPanel.Children.Clear();
      _lineItems.Clear();
      _lastActiveIndex = -1;
    }

    /// <summary>Called from SeekTimer_Tick (~50 ms). positionMs is song position in ms.</summary>
    public void UpdatePosition(double positionMs) {
      if (_lineItems.Count == 0) {
        return;
      }

      // Binary search — last line whose timeMs <= positionMs
      int lo = 0, hi = _lineItems.Count - 1, activeIdx = -1;
      while (lo <= hi) {
        int mid = (lo + hi) / 2;
        if (_lineItems[mid].TimeMs <= positionMs) { activeIdx = mid; lo = mid + 1; } else {
          hi = mid - 1;
        }
      }

      // Per-word L-R sweep for the active word-synced line (runs every tick)
      if (activeIdx >= 0 && _lineItems[activeIdx].Words is { } words) {
        UpdateWordHighlight(_lineItems[activeIdx], words, positionMs);
      }

      // Opacity animation — only fires when the active line changes
      if (activeIdx == _lastActiveIndex) {
        return;
      }

      if (_lastActiveIndex >= 0 && _lastActiveIndex < _lineItems.Count) {
        AnimateOpacity(_lineItems[_lastActiveIndex].Container, to: 0.50);
      }

      if (activeIdx >= 0) {
        AnimateOpacity(_lineItems[activeIdx].Container, to: 1.00);
        ScrollToLine(activeIdx);
      }

      _lastActiveIndex = activeIdx;
    }

    // ── Word highlight (L-R karaoke sweep) ───────────────────────────────────

    private static void UpdateWordHighlight(LineDisplayItem item, List<(Run Run, long TimeMs)> words, double positionMs) {
      bool foundCurrent = false;

      for (int i = 0; i < words.Count; i++) {
        (Run run, long timeMs) = words[i];
        long nextTimeMs = i + 1 < words.Count ? words[i + 1].TimeMs : timeMs + 800;

        if (timeMs > positionMs) {
          // Future word — dim
          if (run.Foreground != BrushWordFuture) {
            run.Foreground = BrushWordFuture;
          }
        } else if (nextTimeMs > positionMs) {
          // Current word — sweep left-to-right as it's being sung
          foundCurrent = true;

          if (item.CurrentWordIdx != i) {
            // Hand off: previous current word becomes fully bright
            if (item.CurrentWordIdx >= 0 && item.CurrentWordIdx < words.Count) {
              words[item.CurrentWordIdx].Run.Foreground = BrushWordPast;
            }

            // Attach the sweep brush to the new current word
            run.Foreground = item.SweepBrush!;
            item.CurrentWordIdx = i;
          }

          // Move the sweep point: left of t is bright, right of t is dim
          double t = Math.Clamp((positionMs - timeMs) / (nextTimeMs - timeMs), 0, 1);
          item.SweepBrightStop!.Offset = t;
          item.SweepDimStop!.Offset = t;
        } else {
          // Past word — fully bright
          if (run.Foreground != BrushWordPast) {
            run.Foreground = BrushWordPast;
          }
        }
      }

      if (!foundCurrent) {
        item.CurrentWordIdx = -1;
      }
    }

    // ── Build display ─────────────────────────────────────────────────────────

    private void BuildLyricsDisplay() {
      LyricsPanel.Children.Clear();
      _lineItems.Clear();
      _lastActiveIndex = -1;
      LoadingRing.Visibility = Visibility.Collapsed;
      NoLyricsText.Visibility = Visibility.Collapsed;

      if (_lyrics == null) {
        return;
      }

      ParsedLyricsLine[] synced = _lyrics.synced;

      if (synced.Length == 0) {
        if (_lyrics.plain.Length > 0) {
          foreach (string line in _lyrics.plain) {
            LyricsPanel.Children.Add(PlainLine(line));
          }
        } else {
          NoLyricsText.Visibility = Visibility.Visible;
        }
        return;
      }

      ParsedLyricsAgent[] agents = _lyrics.agents ?? [];
      List<ParsedLyricsAgent> personAgents = agents.Where(a => a.agentType == "person").ToList();
      string? secondaryAgentId = personAgents.Count >= 2 ? personAgents[1].id : null;
      HashSet<string> bgAgentIds = agents.Where(a => a.agentType == "other").Select(a => a.id).ToHashSet();

      ParsedLyricsSection[]? sections = _lyrics.sections;
      if (sections is { Length: > 0 }) {
        foreach (ParsedLyricsSection section in sections) {
          LyricsPanel.Children.Add(SectionHeader(section.name));
          foreach (ParsedLyricsLine line in section.lines) {
            AddLine(line, secondaryAgentId, bgAgentIds);
          }
        }
      } else {
        foreach (ParsedLyricsLine line in synced) {
          AddLine(line, secondaryAgentId, bgAgentIds);
        }
      }
    }

    private void AddLine(
      ParsedLyricsLine line,
      string? secondaryAgentId,
      HashSet<string> bgAgentIds) {
      bool isSecondary = secondaryAgentId != null && line.agentId == secondaryAgentId;
      bool isBg = line.agentId != null && bgAgentIds.Contains(line.agentId);

      TextAlignment textAlign = isSecondary ? TextAlignment.Right : TextAlignment.Center;
      FontStyle fontStyle = isBg ? FontStyle.Italic : FontStyle.Normal;

      FrameworkElement el;
      LineDisplayItem item;

      if (line.words is { Length: > 0 }) {
        RichTextBlock rtb = new() {
          HorizontalAlignment = HorizontalAlignment.Stretch,
          Margin = new Thickness(0, 6, 0, 6),
          FontSize = 24,
          FontStyle = fontStyle,
          Opacity = 0.50,
          TextWrapping = TextWrapping.Wrap
        };

        Paragraph para = new() { TextAlignment = textAlign };
        List<(Run Run, long TimeMs)> words = [];

        foreach (ParsedLyricsWord w in line.words) {
          Run run = new() { Text = w.word + " ", Foreground = BrushWordFuture };
          para.Inlines.Add(run);
          words.Add((run, w.timeMs));
        }

        rtb.Blocks.Add(para);
        el = rtb;

        // Build the per-line L-R sweep brush.
        // Stops: [ bright@0, bright@t, dim@t, dim@1 ]
        // Mutating the two middle stops' Offset each tick costs zero allocations.
        GradientStop sweepBrightStop = new() { Color = ColorWordPast, Offset = 0 };
        GradientStop sweepDimStop = new() { Color = ColorWordFuture, Offset = 0 };
        LinearGradientBrush sweepBrush = new() { StartPoint = new Point(0, 0), EndPoint = new Point(1, 0) };
        sweepBrush.GradientStops.Add(new GradientStop { Color = ColorWordPast, Offset = 0 });
        sweepBrush.GradientStops.Add(sweepBrightStop);
        sweepBrush.GradientStops.Add(sweepDimStop);
        sweepBrush.GradientStops.Add(new GradientStop { Color = ColorWordFuture, Offset = 1 });

        item = new LineDisplayItem {
          TimeMs = line.timeMs,
          Container = el,
          Words = words,
          SweepBrush = sweepBrush,
          SweepBrightStop = sweepBrightStop,
          SweepDimStop = sweepDimStop
        };
      } else {
        el = new TextBlock {
          Text = line.line,
          HorizontalAlignment = HorizontalAlignment.Stretch,
          TextAlignment = textAlign,
          Margin = new Thickness(0, 6, 0, 6),
          FontSize = 24,
          FontWeight = new FontWeight { Weight = 600 },
          FontStyle = fontStyle,
          Opacity = 0.50,
          TextWrapping = TextWrapping.Wrap
        };

        item = new LineDisplayItem { TimeMs = line.timeMs, Container = el };
      }

      LyricsPanel.Children.Add(el);
      _lineItems.Add(item);

      if (!string.IsNullOrEmpty(line.translation)) {
        LyricsPanel.Children.Add(new TextBlock {
          Text = line.translation,
          FontSize = 14,
          Opacity = 0.40,
          TextAlignment = textAlign,
          HorizontalAlignment = HorizontalAlignment.Stretch,
          Margin = new Thickness(0, 0, 0, 6),
          TextWrapping = TextWrapping.Wrap
        });
      }
    }

    // ── Static element helpers ────────────────────────────────────────────────

    private static TextBlock PlainLine(string text) {
      return new() {
        Text = text,
        FontSize = 20,
        Opacity = 0.9,
        TextAlignment = TextAlignment.Center,
        HorizontalAlignment = HorizontalAlignment.Stretch,
        TextWrapping = TextWrapping.Wrap,
        Margin = new Thickness(0, 6, 0, 6)
      };
    }

    private static TextBlock SectionHeader(string name) {
      return new() {
        Text = name.ToUpperInvariant(),
        FontSize = 11,
        Opacity = 0.35,
        CharacterSpacing = 150,
        TextAlignment = TextAlignment.Center,
        HorizontalAlignment = HorizontalAlignment.Stretch,
        Margin = new Thickness(0, 28, 0, 10)
      };
    }

    // ── Animations & scrolling ────────────────────────────────────────────────

    private static void AnimateOpacity(UIElement element, double to) {
      DoubleAnimation anim = new() {
        To = to,
        Duration = new Duration(TimeSpan.FromMilliseconds(300)),
        EasingFunction = new QuadraticEase { EasingMode = EasingMode.EaseOut }
      };
      Storyboard.SetTarget(anim, element);
      Storyboard.SetTargetProperty(anim, "Opacity");
      Storyboard sb = new();
      sb.Children.Add(anim);
      sb.Begin();
    }

    private void ScrollToLine(int index) {
      _ = Task.Delay(50); // let layout settle after opacity change
      if (index < 0 || index >= _lineItems.Count) {
        return;
      }

      LineDisplayItem item = _lineItems[index];
      GeneralTransform? transform = item.Container.TransformToVisual(LyricsPanel);
      Point pt = transform.TransformPoint(new Point(0, 0));
      double lineH = item.Container.ActualHeight;
      double target = Math.Max(0, pt.Y - (LyricsScroller.ActualHeight / 2) + (lineH / 2));
      _ = LyricsScroller.ChangeView(null, target, null);
    }
  }
}
