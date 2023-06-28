using System.Security.Claims;
using System.Security.Principal;
using System.Text.Encodings.Web;
using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.Options;

namespace Unilake.Worker.Authentication;

public class LocalAuthHandler : AuthenticationHandler<AuthenticationSchemeOptions>
{
    public LocalAuthHandler(IOptionsMonitor<AuthenticationSchemeOptions> options, ILoggerFactory logger, UrlEncoder encoder, ISystemClock clock) 
        : base(options, logger, encoder, clock)
    {
        
    }

    protected override Task<AuthenticateResult> HandleAuthenticateAsync()
    {
        var remoteIpAddress = Request.HttpContext.Connection.RemoteIpAddress?.ToString() ?? string.Empty;

        if (remoteIpAddress is not ("::1" or "127.0.0.1"))
            return Task.FromResult(AuthenticateResult.Fail("Invalid origin address!"));
        
        var identity = new ClaimsIdentity(new[] { new Claim("ClientID", "Default") }, Scheme.Name);
        var principal = new GenericPrincipal(identity, null);
        var ticket = new AuthenticationTicket(principal, Scheme.Name);
        return Task.FromResult(AuthenticateResult.Success(ticket));
    }
}