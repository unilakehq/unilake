using OneOf;
using OneOf.Types;
using Unilake.Worker.Models.Git;

namespace Unilake.Worker.Services.Git;

public interface IGitService
{
    OneOf<Success, Error<Exception>> Clone(string repositoryUrl, string localPath);
    OneOf<Success, Error<Exception>> Checkout(string branchOrCommit, bool createBranch);
    OneOf<Success, Error<Exception>> Pull(string remote, string branch);
    OneOf<Success, Error<Exception>> Push(string remote, string branch);
    OneOf<Success, Error<Exception>> CreateBranch(string branchName);
    OneOf<Success, Error<Exception>> Commit(string message);
    OneOf<Success, Error<Exception>> DeleteBranch(string branchName);
    OneOf<Success, Error<Exception>> Fetch(string remote);
    OneOf<Success, Error<Exception>> Revert(string[] files);
    OneOf<Success, Error<Exception>> AbortMerge();
    OneOf<Success<string[]>, Error<Exception>> Branches();
    OneOf<Success<string>, Error<Exception>> ActiveBranch();
    OneOf<Success<GitFileDiff[]>, Error<Exception>> GetFileDiff(string sourceBranch, string targetBranch, params string[] paths);
    OneOf<Success<GitDiff[]>, Error<Exception>> GetDiff(string sourceBranch, string targetBranch);
    OneOf<Success, Error<Exception>> Reset();
}