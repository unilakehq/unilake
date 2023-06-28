using System.IO.Abstractions;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Models;
using Unilake.Worker.Models.File;

namespace Unilake.Worker.Services.File;

public class FileService : IFileService
{
    private readonly EnvironmentOptions _environmentOptions;
    private readonly IFileSystem _fileSystem;

    public FileService(EnvironmentOptions environmentOptions, IFileSystem fileSystem)
    {
        _environmentOptions = environmentOptions;
        _fileSystem = fileSystem;
    }

    public OneOf<None, Exception> PutFile(string path, Stream stream)
    {
        try
        {
            using var fileStream = _fileSystem.File.Create(PrefixPath(path));
            stream.CopyTo(fileStream);
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<Stream, Exception> GetFile(string path)
    {
        try
        {
            var fileStream = _fileSystem.File.OpenRead(PrefixPath(path));
            return fileStream;
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<True, Exception> DeleteFile(string path)
    {
        try
        {
            if (_fileSystem.File.Exists(PrefixPath(path)))
                _fileSystem.File.Delete(PrefixPath(path));
            else return new Exception("File does not exist");
            return new True();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<True, Exception> RenameFile(string path, string newPath)
    {
        try
        {
            _fileSystem.File.Move(PrefixPath(path), PrefixPath(newPath));
            return new True();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> CreateDirectory(string path)
    {
        try
        {
            _fileSystem.Directory.CreateDirectory(PrefixPath(path));
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> RenameDirectory(string oldPath, string newPath)
    {
        try
        {
            _fileSystem.Directory.Move(PrefixPath(oldPath), PrefixPath(newPath));
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> DeleteDirectory(string path)
    {
        try
        {
            _fileSystem.Directory.Delete(PrefixPath(path), true);
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> CopyFile(string source, string destination)
    {
        try
        {
            _fileSystem.File.Copy(PrefixPath(source), PrefixPath(destination));
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> MoveFile(string source, string destination)
    {
        try
        {
            string sourceFullPath = Path.Combine(_environmentOptions.WorkingDirectory, source);
            string destinationFullPath = Path.Combine(_environmentOptions.WorkingDirectory, destination);

            if (!_fileSystem.File.Exists(sourceFullPath))
            {
                return new FileNotFoundException($"The file '{source}' was not found.", source);
            }

            if (_fileSystem.File.Exists(destinationFullPath))
            {
                return new InvalidOperationException($"The file '{destination}' already exists.");
            }

            _fileSystem.File.Move(sourceFullPath, destinationFullPath);

            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> CopyDirectory(string source, string destination)
    {
        try
        {
            _fileSystem.Directory.CreateDirectory(PrefixPath(destination));

            foreach (var file in _fileSystem.Directory.GetFiles(PrefixPath(source)))
            {
                var fileName = _fileSystem.Path.GetFileName(file);
                var destFile = _fileSystem.Path.Combine(PrefixPath(destination), fileName);
                _fileSystem.File.Copy(file, destFile, true);
            }

            foreach (var dir in _fileSystem.Directory.GetDirectories(PrefixPath(source)))
            {
                var dirName = _fileSystem.Path.GetFileName(dir);
                var destDir = _fileSystem.Path.Combine(PrefixPath(destination), dirName);
                var copyDirectory = CopyDirectory(dir, destDir);
                if (copyDirectory.IsT1)
                    return copyDirectory;
            }

            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<None, Exception> MoveDirectory(string source, string destination)
    {
        try
        {
            _fileSystem.Directory.Move(PrefixPath(source), PrefixPath(destination));
            return new None();
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<DirectoryMetadata, Exception> GetDirectoryContent(string path)
    {
        try
        {
            path = PrefixPath(path);
            var info = new DirectoryInfo(path);
            return new DirectoryMetadata
            {
                Name = info.Name,
                Files = _fileSystem.Directory.GetFiles(path).Select(f => new FileMetadata
                {
                    Extension = _fileSystem.Path.GetExtension(_fileSystem.Path.Join(path, f)),
                    Path = CleanRootPath(path),
                    Name = _fileSystem.Path.GetFileName(f)
                }).ToArray(),
                Path = CleanRootPath(path),
                SubDirectories = _fileSystem.Directory.GetDirectories(path).Select(d => new DirectoryMetadata
                {
                    Name = new DirectoryInfo(CleanRootPath(d)).Name,
                    Path = CleanRootPath(d),
                    Files = Array.Empty<FileMetadata>(),
                    SubDirectories = Array.Empty<DirectoryMetadata>()
                }).ToArray()
            };
        }
        catch (Exception ex)
        {
            return ex;
        }
    }

    public OneOf<bool, Error> FileExists(string path)
    {
        try
        {
            return _fileSystem.File.Exists(PrefixPath(path));
        }
        catch
        {
            return new Error();
        }
    }

    private string PrefixPath(string path) => _fileSystem.Path.Join(_environmentOptions.WorkingDirectory, path);
    
    private string CleanRootPath(string path) => !string.IsNullOrWhiteSpace(_environmentOptions.WorkingDirectory) 
        ? path.Replace(_environmentOptions.WorkingDirectory, "") 
        : path;
}