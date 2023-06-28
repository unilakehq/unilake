using System.Text.RegularExpressions;
using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class DocParser
{
    public async Task<Dictionary<string, DocMetaData>> CreateDocMetaMap(
        List<dynamic> docs,
        string projectName,
        string rootPath)
    {
        var docMetaMap = new Dictionary<string, DocMetaData>();

        if (docs == null)
        {
            return docMetaMap;
        }

        foreach (var doc in docs)
        {
            string packageName = doc.package_name;
            string docName = packageName == projectName ? doc.name : $"{packageName}.{doc.name}";
            string fullPath = ManifestParser.CreateFullPathForNode(projectName, rootPath, packageName, doc.original_file_path);

            if (string.IsNullOrWhiteSpace(fullPath))
                continue;

            try
            {
                string docFile = await System.IO.File.ReadAllTextAsync(fullPath);
                string[] macroFileLines = docFile.Split('\n');

                for (int index = 0; index < macroFileLines.Length; index++)
                {
                    string currentLine = macroFileLines[index];
                    if (Regex.IsMatch(currentLine, $"docs\\s{doc.name}"))
                    {
                        docMetaMap[docName] = new DocMetaData
                        {
                            Path = fullPath,
                            Line = index,
                            Character = currentLine.IndexOf(doc.name)
                        };
                        break;
                    }
                }
            }
            catch (Exception error)
            {
                Console.WriteLine($"File not found at '{fullPath}', project may need to be recompiled. {error}");
                // TODO: this
                // Terminal.Log($"File not found at '{fullPath}', probably compiled is outdated. {error}");
            }
        }

        return docMetaMap;
    }
}