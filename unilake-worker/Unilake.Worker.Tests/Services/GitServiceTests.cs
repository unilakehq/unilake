using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Tests.Services;

[TestClass]
public class GitServiceTests
{
    [TestMethod]
    [DataRow("https://github.com/dbt-labs/jaffle_shop.git", "jaffle_shop")]
    [DataRow("https://github.com/dbt-labs/jaffle_shop", "jaffle_shop")]
    [DataRow("", "", "Invalid Git clone URL")]
    [DataRow("awdadada12313asd", "", "Invalid Git clone URL")]
    public void TestGetRepositoryName(string input, string expected, string expectedError = null)
    {
        var repoName = GitService.GetRepositoryName(input);
        
        if(string.IsNullOrWhiteSpace(expectedError))
            repoName.AsT0.Should().Be(expected);
        else
        {
            repoName.IsT1.Should().BeTrue();
            repoName.AsT1.Message.Should().Be(expectedError);
        }
    }
}