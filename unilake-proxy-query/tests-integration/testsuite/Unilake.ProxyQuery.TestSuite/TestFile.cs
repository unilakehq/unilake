using System;
using System.Text;

namespace Unilake.ProxyQuery.TestSuite;


public record TestFileEntry(string Query, string ExpectedResult);

public static class TestFile
{
    public static List<TestFileEntry> GetTestFileEntries(string filePath)
    {
        if (!File.Exists(filePath))
        {
            throw new FileNotFoundException($"Test file {filePath} not found.");
        }

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

            if (line.StartsWith("====="))
            {
                if (currentQueryResult.Length > 0)
                {
                    toreturn.Add(new TestFileEntry(currentQuery.ToString(), currentQueryResult.ToString()));
                    currentQuery.Clear();
                    currentQueryResult.Clear();
                }

                inQueryBlock = currentQuery.Length == 0;
                inResultBlock = !inQueryBlock;
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
}
