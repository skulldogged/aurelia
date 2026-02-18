using Microsoft.UI.Xaml.Data;

namespace Aurelia.Converters;

public class ArtistsConverter : IValueConverter {
  public object Convert(object value, Type targetType, object parameter, string language) {
    return value is string[] artists && artists.Length > 0 ? string.Join(", ", artists) : "Unknown Artist";
  }

  public object ConvertBack(object value, Type targetType, object parameter, string language) {
    throw new NotImplementedException();
  }
}
