global using FastEndpoints;

var builder = WebApplication.CreateBuilder();
builder.Services.AddFastEndpoints();

var app = builder.Build();
app.UseFastEndpoints(c =>
    {
        c.Serializer.ResponseSerializer = (rsp, dto, cType, _, ct) =>
        {
            rsp.ContentType = cType;
            return rsp.WriteAsync(Newtonsoft.Json.JsonConvert.SerializeObject(dto), ct);
        };
    }
);
app.Run();