using FastEndpoints;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Endpoints.Git;

namespace Unilake.Worker.Tests.Endpoints.Git;

[TestClass]
public class GitPushTests : GitEndpointTestsBase<Push, GitPushRequest, GitActionResultResponse>
{
    private const string DefaultResponse = "Git push action queued"; 
    
    private GitPushRequest CreateRequest(string remote = "origin", string branch = "master")
        => new ()
        {
            Remote = remote,
            Branch = branch
        };
    
    [TestMethod]
    public async Task GitPush_Succeeded_Response_Is_Not_Null()
    {
        var (endpoint, _, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Succeeded_Response_Is_Not_Null(endpoint, CreateRequest());
    }
    
    [TestMethod]
    public async Task GitPush_Succeeded_Response_Is_Of_Type_GitActionResultResponse()
    {
        var (endpoint, _, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Succeeded_Response_Is_Of_Type_GitActionResultResponse(endpoint, CreateRequest());
    }
    
    [TestMethod]
    public async Task GitPush_Succeeded_Response_Status_Code_Is_200()
    {
        var (endpoint, _, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Succeeded_Response_Status_Code_Is_200(endpoint, CreateRequest());
    }
    
    [TestMethod]
    public async Task GitPush_Succeeded_Response_ProcessReferenceId_Matches()
    {
        var (endpoint, _, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Succeeded_Response_ProcessReferenceId_Matches(endpoint, CreateRequest());
    }
    
    [TestMethod]
    public async Task GitPush_Succeeded_Response_Call_To_Publish_Is_Made()
    {
        var (endpoint, _pm, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Succeeded_Response_Call_To_Publish_Is_Made(endpoint, CreateRequest(), _pm);
    }
    
    [TestMethod]
    public async Task GitPush_Failed_Response_Call_To_Publish_Is_Rejected_400()
    {
        var (endpoint, _pm, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Failed_Response_Call_To_Publish_Is_Rejected_400(endpoint, CreateRequest(), _pm);
    }
    
    [TestMethod]
    public async Task GitPush_Failed_Response_Call_To_Publish_Is_Rejected_Message()
    {
        var (endpoint, _pm, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Failed_Response_Call_To_Publish_Is_Rejected_Message(endpoint, CreateRequest(), _pm);
    }
    
    [TestMethod]
    public async Task GitPush_Failed_Response_Call_To_Publish_Is_Rejected_Validation_Failed()
    {
        var (endpoint, _pm, _) = Default(pm => Factory.Create<Push>(pm), DefaultResponse);
        await Failed_Response_Call_To_Publish_Is_Rejected_Validation_Failed(endpoint, CreateRequest(), _pm);
    }
}