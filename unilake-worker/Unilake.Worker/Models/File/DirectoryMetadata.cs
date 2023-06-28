namespace Unilake.Worker.Models.File;

public class DirectoryMetadata
{
    public string Name { get; set; }

    public string Path { get; set; }

    public FileMetadata[] Files { get; set; }

    public DirectoryMetadata[] SubDirectories { get; set; }
}