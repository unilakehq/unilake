namespace Unilake.ProxyQuery.TestIntegration;

public static class Extensions
{
    public static Int32 GetUnixTimestamp(this DateTime dateTime) =>
        (int)dateTime.Subtract(new DateTime(1970, 1, 1)).TotalSeconds;
}