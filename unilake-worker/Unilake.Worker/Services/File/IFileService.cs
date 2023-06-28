using OneOf;
using OneOf.Types;
using Unilake.Worker.Models.File;

namespace Unilake.Worker.Services.File;

public interface IFileService
{
    OneOf<None, Exception> PutFile(string path, Stream stream);
    OneOf<Stream, Exception> GetFile(string path);
    OneOf<True, Exception> DeleteFile(string path);
    OneOf<True, Exception> RenameFile(string oldPath, string newPath);
    OneOf<None, Exception> CreateDirectory(string path);
    OneOf<None, Exception> RenameDirectory(string oldPath, string newPath);
    OneOf<None, Exception> DeleteDirectory(string path);
    OneOf<None, Exception> CopyFile(string source, string destination);
    OneOf<None, Exception> MoveFile(string source, string destination);
    OneOf<None, Exception> CopyDirectory(string source, string destination);
    OneOf<None, Exception> MoveDirectory(string source, string destination);
    OneOf<DirectoryMetadata, Exception> GetDirectoryContent(string path);
    OneOf<bool, Error> FileExists(string path);
}