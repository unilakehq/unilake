using Unilake.Cli.Config;
using FluentAssertions;

namespace Unilake.Cli.Tests;

[TestClass]
public class ValidateTests
{
    [TestMethod]
    public void Validate_Init_Succeedes()
    {
        var sut = new EnvironmentConfig();
        sut.IsValid().Should().BeFalse();
    }
}
