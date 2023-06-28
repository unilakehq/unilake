namespace Unilake.Worker.Endpoints.Oauth2;

public class Auth : Endpoint<EmptyRequest,EmptyResponse>
{
    public override void Configure()
    {
        Get("/oauth2/auth");
    }

    public override async Task HandleAsync(EmptyRequest req, CancellationToken ct) => 
        await SendAsync(new EmptyResponse(),202, ct);
}