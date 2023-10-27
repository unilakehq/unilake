using System.Reflection;
using FluentAssertions;

namespace Unilake.Cli.Tests.Config;

[TestClass]
public class ParserTests
{
    [TestMethod]
    public void Parser_HappyFlow_FromFilePath_Succeeds()
    {
        string filePath = Path.Combine(Enumerable.Range(0, 4)
            .Aggregate(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location), (current, _) => Directory.GetParent(current)?.FullName ?? current), "Unilake.Cli", "unilake.default.yaml");

        File.Exists(filePath).Should().BeTrue();
        var result = Unilake.Cli.Config.Parser.ParseFromPath(filePath);
        result.Should().NotBeNull();
    }
    
    [TestMethod]
    public void Parser_HappyFlow_FromString_Succeeds()
    {        
        string filePath = Path.Combine(Enumerable.Range(0, 4)
            .Aggregate(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location), (current, _) => Directory.GetParent(current)?.FullName ?? current), "Unilake.Cli", "unilake.default.yaml");
        string fileContents = File.ReadAllText(filePath);
        
        var result = Unilake.Cli.Config.Parser.ParseFromString(fileContents);
        result.Should().NotBeNull();
    }
    
    [TestMethod]
    public void Parser_HappyFlow_FromEmbeddedResource_Succeeds()
    {
        var result = Unilake.Cli.Config.Parser.ParseFromEmbeddedResource("Unilake.Cli.unilake.default.yaml");
        result.Should().NotBeNull();
    }
}