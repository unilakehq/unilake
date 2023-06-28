using FakeItEasy;
using FastEndpoints;
using FluentAssertions;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Services;

namespace Unilake.Worker.Tests.Endpoints.File;

public class FileEndpointTestBase<T, TReq, TResp> : FileEndpointTest
    where T : Endpoint<TReq, TResp>
    where TResp : IRequestResponse
    where TReq : notnull
{
    protected (T, IProcessManager, IRequestResponse) Default(Func<IProcessManager, T> createEndpoint, string defaultResponse)
    {
        // arrange
        var response = CreateResponse<FileActionResultResponse>(defaultResponse, "uid");
        var fakeProcessManager = A.Fake<IProcessManager>();
        A.CallTo(() => fakeProcessManager.GenerateProcessId(A<IRequestResponse>.Ignored)).Returns(response.ProcessReferenceId);
        A.CallTo(() => fakeProcessManager.Status<FileActionResultResponse>(response.ProcessReferenceId))
            .Returns(new Success<FileActionResultResponse>(response));
        return (createEndpoint(fakeProcessManager), fakeProcessManager, response);
    }

    protected async Task Succeeded_Response_Is_Not_Null(T endpoint, TReq request)
    {
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        var response = endpoint.Response;
        
        // assert
        response.Should().NotBeNull();
    }
    
    protected async Task Succeeded_Response_Is_Of_Type_GitActionResultResponse(T endpoint, TReq request)
    {
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        var response = endpoint.Response;
        
        // assert
        response.Should().BeOfType<FileActionResultResponse>();
    }
    
    protected async Task Succeeded_Response_Status_Code_Is_200(T endpoint, TReq request)
    {
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        
        // assert
        endpoint.HttpContext.Response.StatusCode.Should().Be(200);
    }
    
    protected async Task Succeeded_Response_ProcessReferenceId_Matches(T endpoint, TReq request)
    {
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        var response = endpoint.Response;
        
        // assert
        response.ProcessReferenceId.Should().Be(response.ProcessReferenceId);
    }
    
    protected async Task Succeeded_Response_Call_To_Publish_Is_Made(T endpoint, TReq request, IProcessManager fakeProcessManager)
    {
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        
        // assert
        A.CallTo(() =>
            fakeProcessManager.PublishEventAsync(A<FileTaskEvent>.Ignored, A<Mode>.Ignored,
                A<CancellationToken>.Ignored)).MustHaveHappenedOnceExactly();
    }

    protected async Task Failed_Response_Call_To_Publish_Is_Rejected_400(T endpoint, TReq request, IProcessManager fakeProcessManager)
    {
        // arrange
        A.CallTo(() =>
                fakeProcessManager.Status<FileActionResultResponse>(A<string>.Ignored))
            .Returns(new Error<Exception>(new Exception("This action failed")));
        
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        
        // assert
        endpoint.HttpContext.Response.StatusCode.Should().Be(400);
    }
    
    protected async Task Failed_Response_Call_To_Publish_Is_Rejected_Message(T endpoint, TReq request, IProcessManager fakeProcessManager)
    {
        // arrange
        A.CallTo(() =>
                fakeProcessManager.Status<FileActionResultResponse>(A<string>.Ignored))
            .Returns(new Error<Exception>(new Exception("This action failed")));
        
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        
        // assert
        endpoint.ValidationFailures.First().ErrorMessage.Should().Be("This action failed");
    }
    
    protected async Task Failed_Response_Call_To_Publish_Is_Rejected_Validation_Failed(T endpoint, TReq request, IProcessManager fakeProcessManager)
    {
        // arrange
        A.CallTo(() =>
                fakeProcessManager.Status<FileActionResultResponse>(A<string>.Ignored))
            .Returns(new Error<Exception>(new Exception("This action failed")));
        
        // act
        await endpoint.HandleAsync(request, CancellationToken.None);
        
        // assert
        endpoint.ValidationFailed.Should().BeTrue();
    }  
}