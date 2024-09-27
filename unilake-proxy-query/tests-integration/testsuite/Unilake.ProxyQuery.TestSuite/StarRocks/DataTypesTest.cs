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
            var resultSet = new Runner().RunQuery(line.Query);
            line.ExpectedResult.Should().Be(resultSet.Print());
        }
    }
}
