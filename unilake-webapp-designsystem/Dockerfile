FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build

WORKDIR /app
COPY src .
RUN dotnet build -c Release -o /build
RUN dotnet publish -c Release -o /publish

FROM nginx:alpine AS final
WORKDIR /usr/share/nginx/html
COPY --from=build /publish/wwwroot /usr/local/webapp/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf