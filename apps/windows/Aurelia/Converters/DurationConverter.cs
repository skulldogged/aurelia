using Microsoft.UI.Xaml.Data;
using System;

namespace Aurelia;

public class DurationConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        if (value is long ticks)
        {
            var timeSpan = TimeSpan.FromTicks(ticks);
            return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
        }
        if (value is TimeSpan ts)
        {
            return $"{(int)ts.TotalMinutes}:{ts.Seconds:D2}";
        }
        if (value is double seconds)
        {
            var timeSpan = TimeSpan.FromSeconds(seconds);
            return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
        }
        if (value is int intSeconds)
        {
            var timeSpan = TimeSpan.FromSeconds(intSeconds);
            return $"{(int)timeSpan.TotalMinutes}:{timeSpan.Seconds:D2}";
        }
        return "0:00";
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        throw new NotImplementedException();
    }
}
