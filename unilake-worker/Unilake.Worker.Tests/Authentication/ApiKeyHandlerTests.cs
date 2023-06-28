using System.Security.Claims;
using System.Text.Encodings.Web;
using FakeItEasy;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Authentication;

namespace Unilake.Worker.Tests.Authentication;

[TestClass]
public class ApiKeyHandlerTests
{
    private IOptionsMonitor<AuthenticationSchemeOptions> _options;
    private ILoggerFactory _loggerFactory;
    private UrlEncoder _urlEncoder;
    private ISystemClock _systemClock;
    private IConfiguration _config;
    private ApiKeyHandler _apiKeyHandler;

    [TestInitialize]
    public void Setup()
    {
        _options = A.Fake<IOptionsMonitor<AuthenticationSchemeOptions>>();
        _loggerFactory = A.Fake<ILoggerFactory>();
        _urlEncoder = UrlEncoder.Default;
        _systemClock = A.Fake<ISystemClock>();

        var configurationBuilder = new ConfigurationBuilder();
        configurationBuilder.AddInMemoryCollection(new[]
        {
            new KeyValuePair<string, string>("Auth:ApiKey", "test-api-key")
        });

        _config = configurationBuilder.Build();

        _apiKeyHandler = new ApiKeyHandler(_options, _loggerFactory, _urlEncoder, _systemClock, _config);
    }

    [TestMethod]
    public async Task HandleAuthenticateAsync_ValidApiKey_ReturnsSuccess()
    {
        var context = new DefaultHttpContext();
        context.Request.Headers[ApiKeyHandler.ApiKeyHeader] = "test-api-key";
        _apiKeyHandler.InitializeAsync(new AuthenticationScheme("test-scheme", null, typeof(ApiKeyHandler)), context).Wait();

        var result = await _apiKeyHandler.AuthenticateAsync();

        Assert.IsTrue(result.Succeeded);
        Assert.IsNotNull(result.Principal);
        Assert.AreEqual("Default", result.Principal.FindFirstValue("ClientID"));
    }

    [TestMethod]
    public async Task HandleAuthenticateAsync_InvalidApiKey_ReturnsFailure()
    {
        var context = new DefaultHttpContext();
        context.Request.Headers[ApiKeyHandler.ApiKeyHeader] = "invalid-api-key";
        _apiKeyHandler.InitializeAsync(new AuthenticationScheme("test-scheme", null, typeof(ApiKeyHandler)), context).Wait();

        var result = await _apiKeyHandler.AuthenticateAsync();

        Assert.IsFalse(result.Succeeded);
        Assert.AreEqual("Invalid API credentials!", result.Failure.Message);
    }
}
