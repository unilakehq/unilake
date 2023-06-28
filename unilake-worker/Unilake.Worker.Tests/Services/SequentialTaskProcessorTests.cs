using FakeItEasy;
using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Events;
using Unilake.Worker.Services;

namespace Unilake.Worker.Tests.Services
{
    [TestClass]
    public class SequentialTaskProcessorTests
    {
        private SequentialTaskProcessor _processor;

        [TestInitialize]
        public void Setup()
        {
            _processor = new SequentialTaskProcessor();
        }

        [TestMethod]
        public async Task EnqueueTaskAsync_AddsTaskToQueue()
        {
            var task = ("some-id", A.Fake<Func<Task>>());

            await _processor.EnqueueTaskAsync(task);

            var dequeuedTask = await _processor.DequeueTaskAsync(CancellationToken.None);
            dequeuedTask.Should().BeEquivalentTo(task);
        }

        [TestMethod]
        public async Task DequeueTaskAsync_EmptyQueue_WaitsForTask()
        {
            var task = ("some-id", A.Fake<Func<Task>>());
            var dequeueTask = _processor.DequeueTaskAsync(CancellationToken.None);

            Assert.IsFalse(dequeueTask.IsCompleted);
            await _processor.EnqueueTaskAsync(task);
            var dequeuedTask = await dequeueTask;

            dequeuedTask.Should().BeEquivalentTo(task);
        }

        [TestMethod]
        public async Task DequeueTaskAsync_CancellationToken_ThrowsOperationCanceled()
        {
            using var cts = new CancellationTokenSource();
            cts.Cancel();

            Func<Task> act = async () => await _processor.DequeueTaskAsync(cts.Token);

            await act.Should().ThrowAsync<OperationCanceledException>();
        }
    }

    public class SampleServiceTaskEvent : ServiceTaskEvent<int>
    {
        protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(int service) => new Success<IRequestResponse>();
    }
}