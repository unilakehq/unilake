using LibGit2Sharp;
using Microsoft.Extensions.Options;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Models.Git;

namespace Unilake.Worker.Services.Git;

public class GitService : IGitService
{
    private readonly IOptions<GitOptions> _options;
    private string RepositoryPath => _options.Value.RepositoryPath;

    public GitService(IOptions<GitOptions> gitOptions)
    {
        _options = gitOptions;
    }

    public static OneOf<string, Exception> GetRepositoryName(string cloneUrl)
    {
        // Remove any leading or trailing whitespace
        cloneUrl = cloneUrl.Trim();

        // Remove any .git extension from the URL
        if (cloneUrl.EndsWith(".git"))
            cloneUrl = cloneUrl.Substring(0, cloneUrl.Length - 4);

        // Extract the repository name from the URL
        var lastSlashIndex = cloneUrl.LastIndexOf('/');
        if (lastSlashIndex >= 0 && lastSlashIndex < cloneUrl.Length - 1)
            return cloneUrl.Substring(lastSlashIndex + 1);

        // If we couldn't extract the repository name, return an exception
        return new ArgumentException("Invalid Git clone URL");
    }

    public OneOf<Success, Error<Exception>> Clone(string repositoryUrl, string localPath)
    {
        try
        {
            var cloneOptions = new CloneOptions
            {
                CredentialsProvider = (_url, _user, _cred) =>
                    new UsernamePasswordCredentials
                    {
                        Username = _options.Value.AccessToken,
                        Password = string.Empty
                    }
            };

            //_repositoryWrapper.Clone(repositoryUrl, localPath, cloneOptions);
            return new Success();
        }
        catch (Exception exception)
        {
            return new Error<Exception>(exception);
        }
    }

    public OneOf<Success, Error<Exception>> Checkout(string branchOrCommit, bool createBranch)
    {
        try
        {
            using (var repo = new Repository(RepositoryPath))
            {
                Commands.Checkout(repo, branchOrCommit);
                return new Success();
            }
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Pull(string remote, string branch)
    {
        try
        {
            using (var repo = new Repository(RepositoryPath))
            {
                //var remoteBranch = repo.Head.TrackedBranch;

                Commands.Pull(repo, GetSignature(), new PullOptions
                {
                    FetchOptions = new FetchOptions(),
                    MergeOptions = new MergeOptions()
                });
                return new Success();
            }
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Push(string remote, string branch)
    {
        throw new NotImplementedException();
    }

    public OneOf<Success, Error<Exception>> CreateBranch(string branchName)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            var checkout = Checkout(_options.Value.DefaultBranch, false);
            if (checkout.IsT1)
                return checkout.AsT1;
            repo.Branches.Add(branchName, repo.Head.Tip);
            return new Success();
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Commit(string message)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            var signature = GetSignature();
            repo.Commit(message, signature, signature);
            // TODO: fix this
            return Push("", "");
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> DeleteBranch(string branchName)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            repo.Branches.Remove(branchName);
            return new Success();
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Fetch(string remote)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            Commands.Fetch(repo, "origin", new string[0], new FetchOptions { TagFetchMode = TagFetchMode.All }, null);
            return new Success();
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Revert(string[] files)
    {
        // if files is empty, revert all files
        throw new NotImplementedException();
        // using (var repo = new Repository(repositoryPath))
        // {
        //     // Get the current branch tip commit
        //     Commit tipCommit = repo.Head.Tip;

        //     // If filePath is null, revert all files, otherwise revert only the specified file
        //     if (filePath == null)
        //     {
        //         foreach (var entry in repo.RetrieveStatus(new StatusOptions { DetectRenamesInIndex = true }))
        //         {
        //             if (entry.State != FileStatus.Untracked)
        //             {
        //                 repo.CheckoutPaths(tipCommit.Id.Sha, new List<string> { entry.FilePath }, new CheckoutOptions { CheckoutModifiers = CheckoutModifiers.Force });
        //             }
        //         }
        //     }
        //     else
        //     {
        //         // Check if the file exists in the repository
        //         var entry = repo.RetrieveStatus(new StatusOptions { DetectRenamesInIndex = true }).FirstOrDefault(e => e.FilePath == filePath);

        //         if (entry != null && entry.State != FileStatus.Untracked)
        //         {
        //             // Revert the changes in the specified file
        //             repo.CheckoutPaths(tipCommit.Id.Sha,new List<string> { filePath }, new CheckoutOptions { CheckoutModifiers = CheckoutModifiers.Force });
        //         }
        //         else
        //         {
        //         Console.WriteLine("The specified file doesn't exist or is untracked.");
        //         }
        //     }
        // }
    }

    public OneOf<Success<string[]>, Error<Exception>> Branches()
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            var branches = repo.Branches.Select(branch => branch.FriendlyName).ToArray();
            return new Success<string[]>(branches);
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success<string>, Error<Exception>> ActiveBranch()
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            var currentBranch = repo.Head.FriendlyName;
            return new Success<string>(currentBranch);
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success<GitFileDiff[]>, Error<Exception>> GetFileDiff(string sourceBranch, string targetBranch, params string[] paths)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);

            var source = repo.Branches[sourceBranch];
            var target = repo.Branches[targetBranch];

            if (source == null || target == null)
                return new Error<Exception>(new Exception("Source or target branch not found."));

            var changes = repo.Diff.Compare<TreeChanges>(source.Tip.Tree, target.Tip.Tree, paths);
            var changeInfoList = new List<GitFileDiff>();

            foreach (var change in changes)
            {
                string sourceContent = null;
                string targetContent = null;

                if (change.OldOid != null)
                {
                    var oldBlob = repo.Lookup<Blob>(change.OldOid);
                    sourceContent = oldBlob.GetContentText();
                }

                if (change.Oid != null)
                {
                    var newBlob = repo.Lookup<Blob>(change.Oid);
                    targetContent = newBlob.GetContentText();
                }

                changeInfoList.Add(new GitFileDiff
                {
                    OldPath = change.OldPath,
                    NewPath = change.Path,
                    Kind = change.Status,
                    SourceContent = sourceContent,
                    TargetContent = targetContent,
                    ObjectId = change.Oid?.ToString() ?? "unknown",
                });
            }

            return new Success<GitFileDiff[]>(changeInfoList.ToArray());
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }
    public OneOf<Success<GitDiff[]>, Error<Exception>> GetDiff(string sourceBranch, string targetBranch)
    {
        try
        {
            using var repo = new Repository(RepositoryPath);

            var source = repo.Branches[sourceBranch];
            var target = repo.Branches[targetBranch];

            if (source == null || target == null)
                return new Error<Exception>(new Exception("Source or target branch not found."));

            var changes = repo.Diff.Compare<TreeChanges>(source.Tip.Tree, target.Tip.Tree);
            return new Success<GitDiff[]>(changes.Select(x => x as GitDiff).ToArray());
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    public OneOf<Success, Error<Exception>> Reset()
    {
        throw new NotImplementedException();
    }

    public OneOf<Success, Error<Exception>> AbortMerge()
    {
        try
        {
            using var repo = new Repository(RepositoryPath);
            var headCommit = repo.Head.Tip;
            var firstCommitParent = headCommit.Parents.First();
            repo.Reset(ResetMode.Soft, firstCommitParent);
            return new Success();
        }
        catch (Exception ex)
        {
            return new Error<Exception>(ex);
        }
    }

    private Signature GetSignature() => new (_options.Value.Name, _options.Value.Email, DateTimeOffset.Now);
}