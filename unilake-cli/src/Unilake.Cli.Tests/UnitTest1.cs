namespace Unilake.Cli.Tests;

[TestClass]
public class UnitTest1
{
    [TestMethod]
    public void TestMethod1()
    {
        var sometest = 2;
        var someresult = 4;
        Assert.IsTrue(sometest == someresult/2);
    }
}