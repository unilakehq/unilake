using Unilake.WebApp.DesignSystem.Services.State;

namespace Unilake.WebApp.DesignSystem.Services.Messaging;

/// <summary>
/// todo: implement this, we can make use of the following service for now: https://sse.dev/
/// </summary>
public class SseMessageServiceImpl : MessageService
{
    private readonly StateService _stateService;

    public SseMessageServiceImpl(StateService stateService)
    {
        _stateService = stateService;
    }

    public override Task ConnectAsync()
    {
        Console.WriteLine("Connecting to SSE...");
        return Task.CompletedTask;
    }
}