using FakeItEasy;
using FastEndpoints;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Endpoints.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Tests.Endpoints.Activity;

[TestClass]
public class TrackActivityTests
{
    [TestMethod]
    public async Task TrackActivityTests_CallToTrackActivityMustHaveBeenMade()
    {
        // Arrange
        IActivityTracker tracker = A.Fake<IActivityTracker>();
        var endpoint = Factory.Create<TrackActivity>(tracker);
        
        // Act
        await endpoint.HandleAsync(CancellationToken.None);
        
        // Assert
        A.CallTo(() => tracker.TrackActivity()).MustHaveHappenedOnceExactly();
    }
}