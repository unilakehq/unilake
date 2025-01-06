using System.Diagnostics;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Unilake.ProxyQuery.TestSuite.StarRocks;

[TestClass]
public class BenchmarkTests
{
    private static bool _hasError;

    // Send many requests benchmark
    [TestMethod]
    public async Task PerformanceTest()
    {
        var stopwatch = Stopwatch.StartNew();

        Task DoWork()
        {
            var runner = new Runner();
            var query = "SELECT 1";
            for (int i = 0; i < 100; i++)
                try
                {
                    if (_hasError)
                        break;
                    runner.ExecuteQueryDatatable(query);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error executing query: {ex.Message}");
                    _hasError = true;
                }

            return Task.CompletedTask;
        }

        var tasks = new List<Task>();
        for (int i = 0; i < 100; i++)
            tasks.Add(Task.Run(DoWork));

        await Task.WhenAll(tasks);
        stopwatch.Stop();
        Console.WriteLine($"Elapsed time: {stopwatch.ElapsedMilliseconds} ms");
    }
    // Sleep and check query proxy overhead
}