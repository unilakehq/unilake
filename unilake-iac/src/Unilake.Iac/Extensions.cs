using System.Globalization;
using System.Text;

namespace Unilake.Iac;

public static class Extensions
{
    public static string ToRfc3339String(this DateTime dateTime) => dateTime.ToString("yyyy-MM-dd'T'HH:mm:ss.fffzzz", DateTimeFormatInfo.InvariantInfo);

    /// <summary>
    /// Extension method to decode a Base64 string.
    /// </summary>
    /// <param name="base64EncodedData">The encoded data.</param>
    /// <returns>The decoded string.</returns>
    public static string DecodeBase64(this string base64EncodedData)
    {
        byte[] base64EncodedBytes = Convert.FromBase64String(base64EncodedData);
        return Encoding.UTF8.GetString(base64EncodedBytes);
    }

    /// <summary>
    /// Extension method to Base64 encode a string.
    /// </summary>
    /// <param name="plainText">The text to encode</param>
    /// <returns>The encoded string.</returns>
    public static string EncodeBase64(this string plainText)
    {
        byte[] plainTextBytes = Encoding.UTF8.GetBytes(plainText);
        return Convert.ToBase64String(plainTextBytes);
    }
}