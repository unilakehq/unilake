using System.Text.RegularExpressions;
using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class MacroParser
{
    public async Task<Dictionary<string, MacroMetaData>> CreateMacroMetaMap(
        string projectName,
        object[] macros,
        string rootPath)
    {
        var macroMetaMap = new Dictionary<string, MacroMetaData>();

        if (macros == null)
            return macroMetaMap;

        foreach (dynamic macro in macros)
        {
            string packageName = macro.package_name;
            string name = macro.name;
            string originalFilePath = macro.original_file_path;

            string macroName = packageName == projectName ? name : $"{packageName}.{name}";
            string fullPath = ManifestParser.CreateFullPathForNode(projectName, rootPath, packageName, originalFilePath);

            if (string.IsNullOrEmpty(fullPath))
                continue;

            try
            {
                string macroFile = await System.IO.File.ReadAllTextAsync(fullPath);
                string[] macroFileLines = macroFile.Split('\n');

                for (int index = 0; index < macroFileLines.Length; index++)
                {
                    string currentLine = macroFileLines[index];
                    if (Regex.IsMatch(currentLine, $"macro\\s{name}\\("))
                    {
                        macroMetaMap[macroName] = new MacroMetaData
                        {
                            Path = fullPath,
                            Line = index,
                            Character = currentLine.IndexOf(name, StringComparison.Ordinal)
                        };
                        break;
                    }
                }
            }
            catch (Exception error)
            {
                Console.WriteLine($"File not found at '{fullPath}', project may need to be recompiled. {error}");
                // TODO:
                // _terminal.Log($"File not found at '{fullPath}', probably compiled is outdated. {error}");
            }
        }

        return macroMetaMap;
    }
}
