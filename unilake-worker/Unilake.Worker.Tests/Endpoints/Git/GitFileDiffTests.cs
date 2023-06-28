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
public class GitFileDiffTests
{
    private IGitService _gitService;
    private DiffFile _endpoint;

    [TestInitialize]
    public void Setup()
    {
        _gitService = A.Fake<IGitService>();
        _endpoint = Factory.Create<DiffFile>(_gitService);
        _endpoint.Map = new DiffFileMapper();
    }
    
    private GitDiffFileRequest CreateRequest(string sourceBranch = "develop", string targetBranch = "main")
        => new()
        {
            SourceBranch = sourceBranch,
            TargetBranch = targetBranch,
            FilePaths = new[] {"/home/user/file1.txt", "/home/user/file2.txt"},
        };

    private Success<GitFileDiff[]> GetResponse() => new (new[]
    {
        new GitFileDiff
        {
            OldPath = "/some/old/path.txt",
            NewPath = "/some/new/path.txt", 
            Kind = ChangeKind.Modified,
            SourceContent = "source content",
            TargetContent = "target content",
            ObjectId = "some-id",
        }
    });


    [TestMethod]
    public async Task GitDiff_Succeeded_Response_Is_Not_Null()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Should().NotBeNull();
    }

    [TestMethod]
    public async Task GitDiff_Succeeded_Response_Is_Of_Type_GitBranchesResultResponse()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Should().BeOfType<GitDiffOverviewResponse>();
    }

    [TestMethod]
    public async Task GitDiff_Succeeded_Response_Status_Code_Is_200()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(200);
    }

    [TestMethod]
    public async Task GitDiff_Succeeded_Response_Matches()
    {
        // act
        var responseInput = GetResponse();
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(responseInput);
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.Response.Length.Should().Be(responseInput.Value.Length);
        _endpoint.Response[0].Kind.Should().Be(responseInput.Value[0].Kind.ToString());
        _endpoint.Response[0].OldPath.Should().Be(responseInput.Value[0].OldPath);
        _endpoint.Response[0].NewPath.Should().Be(responseInput.Value[0].NewPath);
        _endpoint.Response[0].SourceContent.Should().Be(responseInput.Value[0].SourceContent);
        _endpoint.Response[0].TargetContent.Should().Be(responseInput.Value[0].TargetContent);
    }

    [TestMethod]
    public async Task GitDiff_Succeeded_Response_Call_To_GitService_Is_Made()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).Returns(GetResponse());
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored)).MustHaveHappenedOnceExactly();
    }

    [TestMethod]
    public async Task GitDiff_Failed_Response_Call_To_GitService_Is_Rejected_400()
    {
        // act
        A.CallTo(() => _gitService.GetFileDiff(A<string>.Ignored, A<string>.Ignored))
            .Returns(new Error<Exception>(new Exception("Some error")));
        await _endpoint.HandleAsync(CreateRequest(), CancellationToken.None);

        // assert
        _endpoint.HttpContext.Response.StatusCode.Should().Be(400);
    }
}