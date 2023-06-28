using FakeItEasy;
using FastEndpoints;
using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using OneOf.Types;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Endpoints.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Tests.Endpoints.Git;

[TestClass]
public class GitBranchesTests
{
    private IGitService _gitService;
    private Branches _endpoint;

    [TestInitialize]
    public void Setup()
    {
        _gitService = A.Fake<IGitService>();
        _endpoint = Factory.Create<Branches>(_gitService);
    }

    [TestMethod]
    public async Task GitBranches_Succeeded_Response_Is_Not_Null()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.Response.Should().NotBeNull();
    }

    [TestMethod]
    public async Task GitBranches_Succeeded_Response_Is_Of_Type_GitBranchesResultResponse()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.Response.Should().BeOfType<GitBranchesResponse[]>();
    }

    [TestMethod]
    public async Task GitBranches_Succeeded_Response_Status_Code_Is_200()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(200);
    }

    [TestMethod]
    public async Task GitBranches_Succeeded_Response_Matches()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.Response.Length.Should().Be(2);
        _endpoint.Response[0].Name.Should().Be("main");
        _endpoint.Response[0].IsActive.Should().Be(true);
        _endpoint.Response[1].Name.Should().Be("develop");
        _endpoint.Response[1].IsActive.Should().Be(false);
    }

    [TestMethod]
    public async Task GitBranches_Succeeded_Response_Call_To_GitService_Is_Made()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        A.CallTo(() => _gitService.Branches()).MustHaveHappenedOnceExactly();
        A.CallTo(() => _gitService.ActiveBranch()).MustHaveHappenedOnceExactly();
    }

    [TestMethod]
    public async Task GitBranches_Failed_Response_Call_To_GitService_Is_Rejected_400()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Error<Exception>(new Exception("some error")));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(400);
    }

    [TestMethod]
    public async Task GitBranches_Failed_Response_Call_To_GitService_Is_Rejected_Message_ActiveBranch()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Success<string[]>(new[] { "main", "develop" }));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Error<Exception>(new Exception("some error")));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.ValidationFailures.First().ErrorMessage.Should().Be("Failed to fetch currently active branch");
    }

    [TestMethod]
    public async Task GitBranches_Failed_Response_Call_To_GitService_Is_Rejected_Message_Branches()
    {
        // act
        A.CallTo(() => _gitService.Branches()).Returns(new Error<Exception>(new Exception("Rejected message")));
        A.CallTo(() => _gitService.ActiveBranch()).Returns(new Success<string>("main"));

        await _endpoint.HandleAsync(CancellationToken.None);

        // assert
        _endpoint.ValidationFailures.First().ErrorMessage.Should().Be("Failed to fetch branches");
    }
}