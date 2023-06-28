using FakeItEasy;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests;
using Unilake.Worker.Endpoints.Process;
using Unilake.Worker.Services;

namespace Unilake.Worker.Tests.Endpoints.Process
{
    //[TestClass]
    public class CancelTests
    {
        private IProcessManager _manager;
        private Cancel _cancelEndpoint;

        [TestInitialize]
        public void Setup()
        {
            _manager = A.Fake<IProcessManager>();
            _cancelEndpoint = new Cancel(_manager);
        }

        //[TestMethod]
        public async Task HandleAsync_CancelRequest_Success()
        {
            var request = new CancelRequest { ProcessReferenceId = "test-process-id" };

            IRequestResponse requestResponse = A.Fake<IRequestResponse>();
            A.CallTo(() => _manager.Cancel(request.ProcessReferenceId))
                .Returns(OneOf<Success<IRequestResponse>, Error<Exception>>.FromT0(new Success<IRequestResponse>(requestResponse)));

            await _cancelEndpoint.HandleAsync(request, CancellationToken.None);

            // Verify that the cancellation was successful
            A.CallTo(() => _manager.Cancel(request.ProcessReferenceId)).MustHaveHappenedOnceExactly();
        }

        //[TestMethod]
        public async Task HandleAsync_CancelRequest_Failure()
        {
            var request = new CancelRequest { ProcessReferenceId = "test-process-id" };

            A.CallTo(() => _manager.Cancel(request.ProcessReferenceId))
                .Returns(OneOf<Success<IRequestResponse>, Error<Exception>>.FromT1(new Error<Exception>(new Exception("Test error"))));

            await _cancelEndpoint.HandleAsync(request, CancellationToken.None);

            // Verify that the cancellation failed
            A.CallTo(() => _manager.Cancel(request.ProcessReferenceId)).MustHaveHappenedOnceExactly();
        }
    }
}