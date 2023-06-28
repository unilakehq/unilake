global using FastEndpoints;
using System.IO.Abstractions;
using System.Text.Json.Serialization;
using System.Threading.Channels;
using FastEndpoints.Swagger;
using Microsoft.AspNetCore.Authentication;
using NSwag;
using Prometheus;
using Unilake.Worker.Authentication;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Events.Dbt;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Models;
using Unilake.Worker.Models.Git;
using Unilake.Worker.Services;
using Unilake.Worker.Services.Activity;
using Unilake.Worker.Services.Dbt;
using Unilake.Worker.Services.Dbt.Manifest;
using Unilake.Worker.Services.File;
using Unilake.Worker.Services.Git;

const string schemaNameApiKey = "ApiKey";
const string schemaNameLocalAuth = "LocalAuth";

var builder = WebApplication.CreateBuilder();
builder.Services
    .AddFastEndpoints()
    .AddAuthentication(schemaNameApiKey)
    .AddScheme<AuthenticationSchemeOptions, ApiKeyHandler>(schemaNameApiKey, null)
    .AddScheme<AuthenticationSchemeOptions, LocalAuthHandler>(schemaNameLocalAuth, null);

builder.Services.AddAuthorization(options =>
{
    options.AddPolicy(schemaNameApiKey, policy => policy.RequireAuthenticatedUser());
});

if (builder.Environment.IsDevelopment())
    builder.Services.AddSwaggerDoc(settings =>
    {
        settings.Title = "Unilake Worker Instance";
        settings.Version = "v1";
        settings.AddAuth(schemaNameApiKey, new()
        {
            Name = ApiKeyHandler.ApiKeyHeader,
            In = OpenApiSecurityApiKeyLocation.Header,
            Type = OpenApiSecuritySchemeType.ApiKey,
        });

    }, addJWTBearerAuth: false);


builder.Services.AddSingleton(Channel.CreateUnbounded<EventStreamResponse>(new UnboundedChannelOptions
{
    SingleReader = false,
    SingleWriter = false
}));
builder.Services.AddTransient(svc => svc.GetRequiredService<Channel<EventStreamResponse>>().Reader);
builder.Services.AddTransient(svc => svc.GetRequiredService<Channel<EventStreamResponse>>().Writer);

builder.Services.AddSingleton<IFileSystem, FileSystem>();
builder.Services.AddSingleton<IGitService, GitService>();
builder.Services.AddSingleton<IFileService, FileService>();
builder.Services.AddSingleton<IDbtService, DbtProjectContainer>();
builder.Services.AddSingleton<IProcessManager, ProcessManager>();
builder.Services.AddSingleton<IActivityTracker, ActivityTracker>();
builder.Services.AddSingleton<GitOptions>();
builder.Services.AddSingleton<EnvironmentOptions>();
builder.Services.AddSingleton<SequentialTaskProcessor>();
builder.Services.AddHostedService<SequentialTaskProcessorHostedService<GitTaskEvent>>();
builder.Services.AddHostedService<SequentialTaskProcessorHostedService<FileTaskEvent>>();
builder.Services.AddHostedService<SequentialTaskProcessorHostedService<DbtTaskEvent>>();
builder.Services.AddHostedService<ActivityReporter>();

builder.Services.AddReverseProxy()
    .LoadFromConfig(builder.Configuration.GetSection("ReverseProxy"));

var app = builder.Build();
app.UseAuthentication();
app.UseAuthorization();
app.MapReverseProxy();
app.UseFastEndpoints(c => c.Serializer.Options.Converters.Add(new JsonStringEnumConverter()));
if (app.Environment.IsDevelopment())
{
    app.UseOpenApi();
    app.UseSwaggerUi3(c => c.ConfigureDefaults());
}
var metricServer = new MetricServer(port: 9090);
metricServer.Start();
app.Run();