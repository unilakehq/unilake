namespace Unilake.Worker;

public static class Extensions
{
    public static string FirstToUpper(this string str)
    {
        if (string.IsNullOrEmpty(str))
            return str;
        return char.ToUpper(str[0]) + str[1..];
    }
}