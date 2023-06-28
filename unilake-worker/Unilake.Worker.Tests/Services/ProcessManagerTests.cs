using FastEndpoints;
using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Services;

namespace Unilake.Worker.Tests.Services
{
    [TestClass]
    public class ProcessManagerTests
    {
        private ProcessManager _processManager;

        [TestInitialize]
        public void Setup()
        {
            _processManager = new ProcessManager();
        }

        [TestMethod]
        public void Status_RequestNotFound_ReturnsError()
        {
            var result = _processManager.Status<SampleRequestResponse>("nonexistent_id");
            result.AsT1.Value.Message.Should().Be("Request not found");
        }

        [TestMethod]
        public void Status_RequestTypeMismatch_ReturnsError()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            var result = _processManager.Status<UnexpectedType>(processId);

            result.AsT1.Value.Message.Should().Be("Request is not of expected type");
        }

        [TestMethod]
        public void Status_RequestFound_ReturnsSuccess()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            var result = _processManager.Status<SampleRequestResponse>(processId);
            result.AsT0.Value.Should().Be(request);
        }

        [TestMethod]
        public void SetResultStatus_RequestNotFound_DoesNotThrow()
        {
            Action act = () => _processManager.SetResultStatus("nonexistent_id", ResultStatus.Success);

            act.Should().NotThrow();
        }

        [TestMethod]
        public void SetResultStatus_RequestFound_SetsStatusAndMessage()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            _processManager.SetResultStatus(processId, ResultStatus.Success, "Success message");

            request.Status.Should().Be(ResultStatus.Success);
            request.Message.Should().Be("Success message");
        }

        [TestMethod]
        public void SetSuccessResponse_RequestFound_SetsSuccess()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            var success = new Success<IRequestResponse>(request);
            _processManager.SetSuccessResponse(processId, success);

            request.Status.Should().Be(ResultStatus.Success);
        }

        [TestMethod]
        public void SetErrorResponse_RequestNotFound_DoesNotThrow()
        {
            Action act = () => _processManager.SetErrorResponse("nonexistent_id", new Error<string>("Error message"));

            act.Should().NotThrow();
        }

        [TestMethod]
        public void SetErrorResponse_RequestFound_SetsErrorAndMessage()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            _processManager.SetErrorResponse(processId, new Error<string>("Error message"));

            request.Status.Should().Be(ResultStatus.Error);
            request.Message.Should().Be("Error message");
        }

        [TestMethod]
        public void GenerateProcessId_AddsRequestAndSetsValues()
        {
            var request = new SampleRequestResponse();

            var processId = _processManager.GenerateProcessId(request);

            processId.Should().NotBeNullOrEmpty();
            request.ProcessReferenceId.Should().Be(processId);
            request.Status.Should().Be(ResultStatus.Queued);
        }

        [TestMethod]
        public void Cancel_RequestNotFound_ReturnsError()
        {
            var result = _processManager.Cancel("nonexistent_id");

            result.AsT1.Value.Message.Should().Be("Request not found");
        }

        [TestMethod]
        public void Cancel_RequestAlreadyCancelled_ReturnsError()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);
            _processManager.SetResultStatus(processId, ResultStatus.Cancelled);
            
            var result = _processManager.Cancel(processId);

            result.AsT1.Value.Message.Should().Be("Request is already cancelled");
        }

        [TestMethod]
        public void Cancel_RequestNotQueued_ReturnsError()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);
            _processManager.SetResultStatus(processId, ResultStatus.InProgress);

            var result = _processManager.Cancel(processId);

            result.AsT1.Value.Message.Should().Be("Request is not queued");
        }

        [TestMethod]
        public void Cancel_RequestQueued_ReturnsSuccessAndCancels()
        {
            var request = new SampleRequestResponse();
            var processId = _processManager.GenerateProcessId(request);

            var result = _processManager.Cancel(processId);

            request.Status.Should().Be(ResultStatus.Cancelled);
            request.Message.Should().Be("Pending cancellation");
        }

    }

    public class SampleRequestResponse : IRequestResponse
    {
        public string ProcessReferenceId { get; set; }
        public ResultStatus Status { get; set; }
        public string Message { get; set; }
    }
    public class UnexpectedType : SampleRequestResponse
    {
        
    }
    
    public class SampleEvent : IEvent
    {
    }
}
