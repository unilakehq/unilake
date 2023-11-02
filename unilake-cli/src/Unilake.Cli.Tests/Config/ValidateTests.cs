using System.Reflection;
using FluentAssertions;
using Unilake.Cli.Config;

namespace Unilake.Cli.Tests.Config;

[TestClass]
public class ValidateTests
{
    [TestMethod]
    public void Validate_Init_Succeeds()
    {
        var sut = new EnvironmentConfig();
        sut.IsValid().Should().BeFalse();
    }
    
    [TestMethod]
    public void Validate_Default_Succeeds()
    {
        string filePath = Path.Combine(Enumerable.Range(0, 4)
            .Aggregate(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location), (current, _) => Directory.GetParent(current!)?.FullName ?? current)!, "Unilake.Cli", "unilake.default.yaml");
        
        File.Exists(filePath).Should().BeTrue();
        var parser = Parser.ParseFromPath(filePath);
        var result = parser.IsValid();
        result.Should().BeTrue();
    }
    
    [TestMethod]
    public void Validate_Invalid_Fails()
    {
        string filePath = Path.Combine(Enumerable.Range(0, 4)
            .Aggregate(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location), (current, _) => Directory.GetParent(current!)?.FullName ?? current)!, "Unilake.Cli", "unilake.default.yaml");
        
        File.Exists(filePath).Should().BeTrue();
        string fileContent = File.ReadAllText(filePath);
        fileContent = fileContent.Replace("unilake.com/v1alpha1", "unilake.com/v-1alpha1");
        var parser = Parser.ParseFromString(fileContent);
        var result = parser.IsValid();
        result.Should().BeFalse();
        parser.GetErrors().Length.Should().BeGreaterThan(0);
    }
}
