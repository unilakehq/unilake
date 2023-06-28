namespace Unilake.Worker.Contracts.Responses.File;

public class DirectoryListResponse
{
    public string Path { get; set; }
    public DirectoryListItemResponse[] Files { get; set; }
}

public class DirectoryListItemResponse
{
    public bool IsFile { get; set; }
    public bool IsDirectory { get; set; }
    public string Name { get; set; }
    public string Extension { get; set; }
    public string FilePath { get; set; }
}