using System;
using System.Data;
using System.Text;

namespace Unilake.ProxyQuery.TestSuite;


public record TestFileEntry(string Query, string ExpectedResult);

public static class TestFile
{
    public static List<TestFileEntry> GetTestFileEntries(string filePath)
    {
        if (!File.Exists(filePath))
            throw new FileNotFoundException($"Test file {filePath} not found.");

        var file = File.ReadAllLines(filePath);
        var toreturn = new List<TestFileEntry>();
        var currentQuery = new StringBuilder();
        var currentQueryResult = new StringBuilder();
        bool inQueryBlock = false;
        bool inResultBlock = false;

        foreach (var line in file)
        {
            if (line.StartsWith("#") || string.IsNullOrWhiteSpace(line))
                continue;

            if (line.StartsWith("---------- Input ----------"))
            {
                if (currentQueryResult.Length > 0)
                {
                    toreturn.Add(new TestFileEntry(currentQuery.ToString(), currentQueryResult.ToString()));
                    currentQuery.Clear();
                    currentQueryResult.Clear();
                }

                inQueryBlock = true;
                continue;
            }

            if (line.StartsWith("---------- Output ----------"))
            {
                inResultBlock = true;
                inQueryBlock = false;
                continue;
            }

            if (inQueryBlock)
                currentQuery.AppendLine(line);
            else if (inResultBlock)
                currentQueryResult.AppendLine(line);

        }

        toreturn.Add(new TestFileEntry(currentQuery.ToString(), currentQueryResult.ToString()));
        return toreturn;
    }

    // public static DataTable FromExpectedResult(string textResult)
    // {
    //     var lines = textResult.Split('\n');
    //     var table = new DataTable();
    //
    //     if (lines.Length > 1)
    //     {
    //         var header = lines[0].Split('|').Skip(1).Select(s => s.TrimEnd(' ')).ToArray();
    //         table.Columns.AddRange(header.Select(s => new DataColumn(s)).ToArray());
    //
    //         for (int i = 1; i < lines.Length; i++)
    //         {
    //             var rowValues = lines[i].Split('|').Skip(1).Select(s => s.TrimEnd(' ')).ToArray();
    //             table.Rows.Add(rowValues);
    //         }
    //     }
    //
    //     return table;
    // }
}
