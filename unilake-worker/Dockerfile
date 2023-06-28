# Build stage
FROM mcr.microsoft.com/dotnet/sdk:7.0 AS build
WORKDIR /app

# Copy the project files and restore dependencies
COPY . .
RUN dotnet restore

# Build the project
RUN dotnet build --configuration Release --no-restore

# Run the unit tests
RUN dotnet test --logger:trx --no-build --verbosity=normal

# Publish the application
RUN dotnet publish --configuration Release --no-build --no-restore --output /app/publish

# Release stage
FROM mcr.microsoft.com/dotnet/aspnet:7.0 AS release
WORKDIR /app

# Copy the published files from the build stage
COPY --from=build /app/publish .

# Set environment variables
ENV ASPNETCORE_URLS=http://+:5000
ENV ASPNETCORE_ENVIRONMENT=Production

# Expose the port
EXPOSE 5000

# Run the application
ENTRYPOINT ["dotnet", "Unilake.Worker.dll"]
