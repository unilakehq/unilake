using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Models.File;

namespace Unilake.Worker.Mappers.File;

public class DirectoryContentMapper : Mapper<DirectoryListRequest, DirectoryListResponse, DirectoryMetadata>
{
    public override DirectoryListResponse FromEntity(DirectoryMetadata e)
    {
        return new DirectoryListResponse
        {
            Path = e.Path,
            Files = e.Files.Select(f => new DirectoryListItemResponse
                {
                    Extension = f.Extension,
                    Name = f.Name,
                    IsDirectory = false,
                    FilePath = f.Path,
                    IsFile = true
                })
                .Concat(e.SubDirectories.Select(d => new DirectoryListItemResponse
                {
                    Extension = string.Empty,
                    FilePath = d.Path,
                    Name = d.Name,
                    IsDirectory = true,
                    IsFile = false
                })).ToArray()
        };
    }
}