using System.IO.Abstractions;
using System.IO.Abstractions.TestingHelpers;
using FluentAssertions;
using Microsoft.Extensions.Configuration;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Models;
using Unilake.Worker.Models.File;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Tests.Services;

[TestClass]
public class FileServiceTests
{
    private IFileService _fileService;
    private EnvironmentOptions _environmentOptions;
    private IFileSystem _fileSystem;

    [TestInitialize]
    public void SetUp()
    {
        string workingDirectory = "/data";
        var inMemorySettings = new Dictionary<string, string> {
            {"environment.workingdirectory", workingDirectory},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();
        
        _environmentOptions = new EnvironmentOptions(configuration);
        _fileSystem = new MockFileSystem();
        _fileService = new FileService(_environmentOptions, _fileSystem);
        _fileSystem.Directory.CreateDirectory(workingDirectory);
    }

    [TestMethod]
    public void GetFileAsync_RetrievesFile()
    {
        // Arrange
        var path = "testFile.txt";
        var content = "Test content";
        using var createStream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        _fileService.PutFile(path, createStream);
        
        // Act
        var result = _fileService.GetFile(path);

        // Assert
        result.IsT0.Should().Be(true);
        using var reader = new StreamReader(result.AsT0);
        var retrievedContent = reader.ReadToEnd();
        retrievedContent.Should().Be(content);
    }
    
    [TestMethod]
    public void GetFileAsync_ThrowsExceptionWhenFileNotFound()
    {
        // Arrange
        var path = "nonExistentFile.txt";

        // Act
        var result = _fileService.GetFile(path);

        // Assert
        result.IsT1.Should().Be(true);
    }
    
    [TestMethod]
    public void DeleteFile_DeletesFile()
    {
        // Arrange
        var path = "testFile.txt";
        var content = "Test content";
        using var createStream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        _fileService.PutFile(path, createStream);

        // Act
        var result = _fileService.DeleteFile(path);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, path)).Should().BeFalse();
    }

    [TestMethod]
    public void DeleteFile_ThrowsExceptionWhenFileNotFound()
    {
        // Arrange
        var path = "nonExistentFile.txt";

        // Act
        var result = _fileService.DeleteFile(path);

        // Assert
        result.IsT1.Should().Be(true);
    }

    [TestMethod]
    public void RenameFile_RenamesFile()
    {
        // Arrange
        var oldPath = "testFile.txt";
        var newPath = "renamedTestFile.txt";
        var content = "Test content";
        using var createStream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        _fileService.PutFile(oldPath, createStream);

        // Act
        var result = _fileService.RenameFile(oldPath, newPath);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, oldPath)).Should().BeFalse();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, newPath)).Should().BeTrue();
    }

    [TestMethod]
    public void RenameFile_ThrowsExceptionWhenFileNotFound()
    {
        // Arrange
        var oldPath = "nonExistentFile.txt";
        var newPath = "newFileName.txt";

        // Act
        var result = _fileService.RenameFile(oldPath, newPath);

        // Assert
        result.IsT1.Should().Be(true);
    }

    [TestMethod]
    public void CreateDirectory_CreatesDirectory()
    {
        // Arrange
        var path = "testDirectory";

        // Act
        var result = _fileService.CreateDirectory(path);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, path)).Should().BeTrue();
    }

    [TestMethod]
    public void RenameDirectory_RenamesDirectory()
    {
        // Arrange
        var oldPath = "testDirectory";
        var newPath = "renamedTestDirectory";
        _fileService.CreateDirectory(oldPath);

        // Act
        var result = _fileService.RenameDirectory(oldPath, newPath);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, oldPath)).Should().BeFalse();
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, newPath)).Should().BeTrue();
    }

    [TestMethod]
    public void RenameDirectory_ThrowsExceptionWhenDirectoryNotFound()
    {
        // Arrange
        var oldPath = "nonExistentDirectory";
        var newPath = "newDirectoryName";

        // Act
        var result = _fileService.RenameDirectory(oldPath, newPath);

        // Assert
        result.IsT1.Should().Be(true);
    }
    
    [TestMethod]
    public void DeleteDirectory_DeletesDirectory()
    {
        // Arrange
        var path = "testDirectory";
        _fileService.CreateDirectory(path);

        // Act
        var result = _fileService.DeleteDirectory(path);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, path)).Should().BeFalse();
    }

    [TestMethod]
    public void DeleteDirectory_ThrowsExceptionWhenDirectoryNotFound()
    {
        // Arrange
        var path = "nonExistentDirectory";

        // Act
        var result = _fileService.DeleteDirectory(path);

        // Assert
        result.IsT1.Should().Be(true);
    }

    [TestMethod]
    public void CopyFile_CopiesFile()
    {
        // Arrange
        var source = "testFile.txt";
        var destination = "copiedTestFile.txt";
        var content = "Test content";
        using var createStream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        _fileService.PutFile(source, createStream);

        // Act
        var result = _fileService.CopyFile(source, destination);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, source)).Should().BeTrue();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination)).Should().BeTrue();
    }

    [TestMethod]
    public void CopyFile_ThrowsExceptionWhenFileNotFound()
    {
        // Arrange
        var source = "nonExistentFile.txt";
        var destination = "newFileName.txt";

        // Act
        var result = _fileService.CopyFile(source, destination);

        // Assert
        result.IsT1.Should().Be(true);
    }

    [TestMethod]
    public void MoveFile_MovesFile()
    {
        // Arrange
        var source = "testFile.txt";
        var destination = "newTestFile.txt";
        var content = "Test content";
        using var stream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        _fileService.PutFile(source, stream);

        // Act
        var result = _fileService.MoveFile(source, destination);

        // Assert
        result.IsT0.Should().BeTrue();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, source)).Should().BeFalse();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination)).Should().BeTrue();
    }

    [TestMethod]
    public void MoveFile_ThrowsExceptionWhenFileNotFound()
    {
        // Arrange
        var source = "nonExistentFile.txt";
        var destination = "newTestFile.txt";

        // Act
        var result = _fileService.MoveFile(source, destination);

        // Assert
        result.IsT1.Should().BeTrue();
        result.AsT1.Should().BeOfType<FileNotFoundException>();
    }
    
    [TestMethod]
    public void MoveDirectory_MovesDirectory()
    {
        // Arrange
        var source = "testDirectory";
        var destination = "newTestDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, source));
        _fileSystem.File.WriteAllText(Path.Combine(_environmentOptions.WorkingDirectory, source, "testFile.txt"), "Test content");

        // Act
        var result = _fileService.MoveDirectory(source, destination);

        // Assert
        result.IsT0.Should().BeTrue();
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination)).Should().BeTrue();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination, "testFile.txt")).Should().BeTrue();
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, source)).Should().BeFalse();
    }

    [TestMethod]
    public void GetDirectoryContent_ReturnsDirectoryMetadataObject()
    {
        // Arrange
        var path = "testDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, path));

        // Act
        var result = _fileService.GetDirectoryContent(path);

        // Assert
        result.IsT0.Should().BeTrue();
        result.AsT0.Should().BeOfType<DirectoryMetadata>();
    }

    [TestMethod]
    public void GetDirectoryContent_ReturnsDirectoryMetadataWithName()
    {
        // Arrange
        var path = "testDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, path));

        // Act
        var result = _fileService.GetDirectoryContent(path);

        // Assert
        result.IsT0.Should().BeTrue();
        var metadata = result.AsT0;
        metadata.Name.Should().Be("testDirectory");
    }

    [TestMethod]
    public void GetDirectoryContent_ReturnsDirectoryMetadataWithPath()
    {
        // Arrange
        var path = "testDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, path));

        // Act
        var result = _fileService.GetDirectoryContent(path);

        // Assert
        result.IsT0.Should().BeTrue();
        var metadata = result.AsT0;
        metadata.Path.Should().Be("/testDirectory");
    }

    [TestMethod]
    public void GetDirectoryContent_ReturnsDirectoryMetadataWithFiles()
    {
        // Arrange
        var path = "testDirectory";
        var fileName = "testFile.txt";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, path));
        _fileSystem.File.WriteAllText(Path.Combine(_environmentOptions.WorkingDirectory, path, fileName), "Test content");

        // Act
        var result = _fileService.GetDirectoryContent(path);

        // Assert
        result.IsT0.Should().BeTrue();
        var metadata = result.AsT0;
        metadata.Files.Should().HaveCount(1);
        var fileMetadata = metadata.Files[0];
        fileMetadata.Name.Should().Be(fileName);
        fileMetadata.Extension.Should().Be(".txt");
        fileMetadata.Path.Should().Be("/testDirectory");
    }

    [TestMethod]
    public void GetDirectoryContent_ReturnsDirectoryMetadataWithSubDirectories()
    {
        // Arrange
        var path = "testDirectory";
        var subDirectoryName = "testSubDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, path, subDirectoryName));

        // Act
        var result = _fileService.GetDirectoryContent(path);

        // Assert
        result.IsT0.Should().BeTrue();
        var metadata = result.AsT0;
        metadata.SubDirectories.Should().HaveCount(1);
        var subDirectoryMetadata = metadata.SubDirectories[0];
        subDirectoryMetadata.Name.Should().Be("testSubDirectory");
        subDirectoryMetadata.Path.Should().Be("/testDirectory/testSubDirectory");
        subDirectoryMetadata.Files.Should().HaveCount(0);
        subDirectoryMetadata.SubDirectories.Should().HaveCount(0);
    }

    [TestMethod]
    public void FileExists_ReturnsFalseWhenFileDoesNotExist()
    {
        // Arrange
        var filePath = "nonExistentFile.txt";

        // Act
        var result = _fileService.FileExists(filePath);

        // Assert
        result.IsT0.Should().BeTrue();
        result.AsT0.Should().BeFalse();
    }

    [TestMethod]
    public void FileExists_ReturnsTrueWhenFileExists()
    {
        // Arrange
        var filePath = "testFile.txt";
        _fileSystem.File.WriteAllText(Path.Combine(_environmentOptions.WorkingDirectory, filePath), "Test content");

        // Act
        var result = _fileService.FileExists(filePath);

        // Assert
        result.IsT0.Should().BeTrue();
    }

    
    [TestMethod]
    public void GetDirectoryContent_ThrowsExceptionWhenDirectoryNotFound()
    {
        // Arrange
        var directoryPath = "nonExistentDirectory";

        // Act
        var result = _fileService.GetDirectoryContent(directoryPath);

        // Assert
        result.IsT1.Should().BeTrue();
        result.AsT1.Should().BeOfType<DirectoryNotFoundException>();
    }
    
    
    [TestMethod]
    public void MoveDirectory_ThrowsExceptionWhenDirectoryNotFound()
    {
        // Arrange
        var source = "nonExistentDirectory";
        var destination = "newTestDirectory";

        // Act
        var result = _fileService.MoveDirectory(source, destination);

        // Assert
        result.IsT1.Should().BeTrue();
        result.AsT1.Should().BeOfType<DirectoryNotFoundException>();
    }

    
    [TestMethod]
    public void CopyDirectory_ThrowsExceptionWhenDirectoryNotFound()
    {
        // Arrange
        var source = "nonExistentDirectory";
        var destination = "newTestDirectory";

        // Act
        var result = _fileService.CopyDirectory(source, destination);

        // Assert
        result.IsT1.Should().BeTrue();
        result.AsT1.Should().BeOfType<DirectoryNotFoundException>();
    }


    [TestMethod]
    public void CopyDirectory_CopiesDirectory()
    {
        // Arrange
        var source = "testDirectory";
        var destination = "newTestDirectory";
        _fileSystem.Directory.CreateDirectory(Path.Combine(_environmentOptions.WorkingDirectory, source));
        _fileSystem.File.WriteAllText(Path.Combine(_environmentOptions.WorkingDirectory, source, "testFile.txt"), "Test content");

        // Act
        var result = _fileService.CopyDirectory(source, destination);

        // Assert
        result.IsT0.Should().BeTrue();
        _fileSystem.Directory.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination)).Should().BeTrue();
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, destination, "testFile.txt")).Should().BeTrue();
    }

    
    [TestMethod]
    public void PutFile_CreatesFile()
    {
        // Arrange
        var path = "testFile.txt";
        var content = "Test content";
        using var stream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));

        // Act
        var result = _fileService.PutFile(path, stream);

        // Assert
        result.IsT0.Should().Be(true);
        _fileSystem.File.Exists(Path.Combine(_environmentOptions.WorkingDirectory, path)).Should().BeTrue();
    }
}