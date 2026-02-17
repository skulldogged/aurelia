using Microsoft.UI.Xaml.Data;
using System;
using System.Linq;

namespace Aurelia;

public class ArtistsConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        if (value is string[] artists && artists.Length > 0)
        {
            return string.Join(", ", artists);
        }
        return "Unknown Artist";
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        throw new NotImplementedException();
    }
}
