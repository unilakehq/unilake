using System.Security.Claims;
using System.Security.Principal;
using System.Text.Encodings.Web;
using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.Options;

namespace Unilake.Worker.Authentication;

public class ApiKeyHandler : AuthenticationHandler<AuthenticationSchemeOptions>
{
    public const string ApiKeyHeader = "x-api-key";
    private readonly string _apiKey;

    public ApiKeyHandler(IOptionsMonitor<AuthenticationSchemeOptions> options,
        ILoggerFactory logger,
        UrlEncoder encoder,
        ISystemClock clock,
        IConfiguration config) : base(options, logger, encoder, clock)
    {
        _apiKey = config["Auth:ApiKey"]
                  ?? throw new InvalidOperationException("Api key not set in appsettings.json");
    }

    protected override Task<AuthenticateResult> HandleAuthenticateAsync()
    {
        Request.Headers.TryGetValue(ApiKeyHeader, out var extractedApiKey);

        if (!extractedApiKey.Equals(_apiKey))
            return Task.FromResult(AuthenticateResult.Fail("Invalid API credentials!"));

        var identity = new ClaimsIdentity(new[] { new Claim("ClientID", "Default") }, Scheme.Name);
        var principal = new GenericPrincipal(identity, null);
        var ticket = new AuthenticationTicket(principal, Scheme.Name);
        return Task.FromResult(AuthenticateResult.Success(ticket));
    }
}