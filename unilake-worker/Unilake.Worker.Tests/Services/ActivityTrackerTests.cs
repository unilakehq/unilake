using FakeItEasy;
using FluentAssertions;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Models.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Tests.Services;

[TestClass]
public class ActivityTrackerTests
{
    private class TestableActivityTracker : ActivityTracker
    {
        private readonly Queue<long> _returnsequence = new ();
        public TestableActivityTracker(IConfiguration configuration, ILogger<ActivityTracker> logger)
            : base(configuration, logger)
        {
        }

        protected override long GetCurrentUnixTimestampUtc() => _returnsequence.Dequeue();

        public void GetCurrentUnixTimestampUtc_Return_Sequence(params long[] values)
        {
            foreach (var i in values)
                _returnsequence.Enqueue(i);
        }
    }
    
    [TestMethod]
    public void Constructor_ValidConfigurationAndLoggerInput()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();
    
        var logger = A.Fake<ILogger<ActivityTracker>>();

        // Act
        var tracker = new TestableActivityTracker(configuration, logger);

        // Assert
        tracker.Should().NotBeNull();
    }
    
    [TestMethod]
    public void TrackActivity_FirstActivityNotSet()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1, 1, 1);

        // Act
        tracker.TrackActivity();
        var status = tracker.GetStatus();

        // Assert
        status.FirstActivityUnixTimestampUtc.Should().Be(1);
    }
    
    [TestMethod]
    public void TrackActivity_FirstActivityAlreadySet()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1, 1, 1);

        // Act
        tracker.TrackActivity(); // First call to set _firstActivity
        var firstStatus = tracker.GetStatus();
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(2, 2, 2, 2);
        tracker.TrackActivity(); // Second call to check if _firstActivity remains unchanged
        var secondStatus = tracker.GetStatus();

        // Assert
        firstStatus.FirstActivityUnixTimestampUtc.Should().Be(1);
        secondStatus.FirstActivityUnixTimestampUtc.Should().Be(1);
    }

    [TestMethod]
    public void TrackActivity_InstanceStatePendingShutdown()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1);

        // Act
        tracker.TrackActivity(); // First call to set _firstActivity
        var firstStatus = tracker.GetStatus();
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1000, 1000); // Force PendingShutdown state
        tracker.TrackActivity(); // Second call when instance state is PendingShutdown
        var secondStatus = tracker.GetStatus();

        // Assert
        firstStatus.InstanceState.Should().Be(InstanceState.Running);
        secondStatus.InstanceState.Should().Be(InstanceState.PendingShutdown);
        secondStatus.LastActivityUnixTimestampUtc.Should().Be(firstStatus.LastActivityUnixTimestampUtc);
    }

    [TestMethod]
    public void GetStatus_FirstActivityNotSet_InstanceStateRunning()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1, 1, 1);
        
        // Act
        var status = tracker.GetStatus();

        // Assert
        status.FirstActivityUnixTimestampUtc.Should().Be(0);
        status.InstanceState.Should().Be(InstanceState.Running);
    }

    [TestMethod]
    public void GetStatus_FirstActivitySet_InstanceStateRunning()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1, 1, 1);

        // Act
        tracker.TrackActivity();
        var status = tracker.GetStatus();

        // Assert
        status.FirstActivityUnixTimestampUtc.Should().Be(1);
        status.InstanceState.Should().Be(InstanceState.Running);
    }

    [TestMethod]
    public void GetStatus_FirstActivitySet_InstanceStatePendingShutdown()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 300); // 300 is greater than _currentShutdownTime

        // Act
        tracker.TrackActivity();
        var status = tracker.GetStatus();

        // Assert
        status.FirstActivityUnixTimestampUtc.Should().Be(1);
        status.InstanceState.Should().Be(InstanceState.PendingShutdown);
    }

    
    [TestMethod]
    public void GetStatus_ReturnsCorrectStatus()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();
        
        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(100, 150, 150);

        // Act
        tracker.TrackActivity();
        tracker.TrackActivity();
        var status = tracker.GetStatus();

        // Assert
        status.FirstActivityUnixTimestampUtc.Should().Be(100);
        status.LastActivityUnixTimestampUtc.Should().Be(150);
        status.ShutdownTimeoutInSeconds.Should().Be(60);
        status.TimeLeftInSeconds.Should().Be(70);
        status.InstanceState.Should().Be(InstanceState.Running);
    }
    
    [TestMethod]
    [DataRow(20, 140, DisplayName= "Positive Adjustment")]
    [DataRow(-20, 100, DisplayName= "Negative Adjustment")]
    [DataRow(0, 120, DisplayName= "Zero Adjustment")]
    public void AdjustTimeout_Positive(int adjustment, int expectedTimeout)
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(1, 1, 1, 11);

        // Act
        tracker.TrackActivity();
        tracker.AdjustTimeout(adjustment);
        var afterAdjustStatus = tracker.GetStatus();

        // Assert
        afterAdjustStatus.TimeLeftInSeconds.Should().Be(expectedTimeout);
    }
    
    [TestMethod]
    public void GetStatus_VariousTimeElapsedValues()
    {
        // Arrange
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:ShutdownTimeoutInSeconds", "60"},
            {"Environment:ShutdownTimePeriodInSeconds", "120"},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<ActivityTracker>>();
        var tracker = new TestableActivityTracker(configuration, logger);
        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(0);

        // Act & Assert
        tracker.TrackActivity();

        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(30);
        var status1 = tracker.GetStatus();
        status1.TimeLeftInSeconds.Should().Be(90);

        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(61);
        var status2 = tracker.GetStatus();
        status2.TimeLeftInSeconds.Should().Be(59);

        tracker.GetCurrentUnixTimestampUtc_Return_Sequence(121);
        var status3 = tracker.GetStatus();
        status3.TimeLeftInSeconds.Should().Be(0);
    }
}