using FakeItEasy;
using FastEndpoints;
using FluentAssertions;
using LibGit2Sharp;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using OneOf.Types;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Endpoints.Git;
using Unilake.Worker.Mappers.Git;
using Unilake.Worker.Models.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Tests.Endpoints.Git;

// TODO: properly implement these unit tests
[Ignore("To be properly implemented")]
[TestClass]
public class GitDiffOverviewTests
{
    private IGitService _gitService;
    private DiffOverview _endpoint;

    [TestInitialize]
    public void Setup()
    {
        _gitService = A.Fake<IGitService>();
        _endpoint = Factory.Create<DiffOverview>(_gitService);
        _endpoint.Map = new DiffOverviewMapper();
    }
    
    private GitDiffOverviewRequest CreateRequest(string sourceBranch = "develop", string targetBranch = "main")
        => new()
        {
            SourceBranch = sourceBranch,
            TargetBranch = targetBranch
        };

    private Success<GitFileDiff[]> GetResponse() => new (new[]
    {
        new GitFileDiff
        {
            Kind = ChangeKind.Added,
            NewPath = "file1.txt",
            OldPath = "file1.txt",
            SourceContent = "Content_1",
            TargetContent = "Content_2",
            ObjectId = ObjectId.Zero.ToString()
        }
    });


    [TestMethod]
    public async Task GitDiffOverview_Succeeded_Response_Is_Not_Null()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Should().NotBeNull();
    }

    [TestMethod]
    public async Task GitDiffOverview_Succeeded_Response_Is_Of_Type_GitBranchesResultResponse()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Should().BeOfType<GitDiffOverviewResponse>();
    }

    [TestMethod]
    public async Task GitDiffOverview_Succeeded_Response_Status_Code_Is_200()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(200);
    }

    [TestMethod]
    public async Task GitDiffOverview_Succeeded_Response_Matches()
    {
        // arrange
        var response = GetResponse();
        
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(response);
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Length.Should().Be(response.Value.Length);
        _endpoint.Response[0].Kind.Should().Be(response.Value[0].Kind.ToString());
        _endpoint.Response[0].FilePath.Should().Be(response.Value[0].OldPath);
    }

    [TestMethod]
    public async Task GitDiffOverview_Succeeded_Response_Call_To_GitService_Is_Made()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).MustHaveHappenedOnceExactly();
    }

    [TestMethod]
    public async Task GitDiffOverview_Failed_Response_Call_To_GitService_Is_Rejected_400()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored))
            .Returns(new Error<Exception>(new Exception("Some error")));
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(400);
    }
}