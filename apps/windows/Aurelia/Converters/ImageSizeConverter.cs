using Microsoft.UI.Xaml.Data;
using System;

namespace Aurelia;

/// <summary>
/// Appends MaxWidth and Quality params to Jellyfin image URLs.
/// Pass the desired pixel width as ConverterParameter.
/// </summary>
public class ImageSizeConverter : IValueConverter
{
    public object? Convert(object value, Type targetType, object parameter, string language)
    {
        if (value is not string url || string.IsNullOrEmpty(url))
            return null;

        var maxWidth = 300;
        if (parameter is string s && int.TryParse(s, out var parsed))
            maxWidth = parsed;

        var separator = url.Contains('?') ? "&" : "?";
        return $"{url}{separator}MaxWidth={maxWidth}&Quality=80";
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        throw new NotImplementedException();
    }
}
