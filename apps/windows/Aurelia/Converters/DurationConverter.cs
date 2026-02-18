using Microsoft.UI.Xaml.Data;

namespace Aurelia.Converters;

public class DurationConverter : IValueConverter {
  public object Convert(object value, Type targetType, object parameter, string language) {
    if (value is long ticks) {
      TimeSpan timeSpan = TimeSpan.FromTicks(ticks);
      return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
    }
    if (value is TimeSpan ts) {
      return $"{(int)ts.TotalMinutes}:{ts.Seconds:D2}";
    }
    if (value is double seconds) {
      TimeSpan timeSpan = TimeSpan.FromSeconds(seconds);
      return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
    }
    if (value is int intSeconds) {
      TimeSpan timeSpan = TimeSpan.FromSeconds(intSeconds);
      return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
    }
    return "0:00";
  }

  public object ConvertBack(object value, Type targetType, object parameter, string language) {
    throw new NotImplementedException();
  }
}
