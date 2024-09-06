using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Unilake.ProxyQuery.TestSuite.StarRocks;

[TestClass]
public class DataTypesTest
{
    [TestMethod]
    public void TestDataTypes()
    {
        var lines = TestFile.GetTestFileEntries("StarRocks/datatypes.txt");

        foreach (var line in lines)
        {
            try
            {
                var result_set = new Runner().RunQuery(line.Query);
                result_set.Should().NotBeNull();
                result_set[0].Should().BeEquivalentTo(line.ExpectedResult);
            }
            catch (Exception e)
            {
                Console.WriteLine($"Error executing query: {line.Query}, Error: {e.Message}");
                throw;
            }
        }
    }
}
