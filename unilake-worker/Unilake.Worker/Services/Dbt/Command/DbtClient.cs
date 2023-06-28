using System.Text.RegularExpressions;
using System.Threading.Channels;
using CliWrap;
using OneOf;
using OneOf.Types;
using Python.Runtime;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Services.Dbt.Command;

public class DbtClient
{
    private bool _isCommandRunning;
    private const int MessageBatchSize = 10;
    private readonly Queue<string> _messages = new();
    private readonly IPythonEnvironment _pythonEnvironment;
    private readonly ChannelWriter<EventStreamResponse> _writer;
    private readonly ILogger _logger;
    private readonly Dictionary<string, (string, string)> _products = new();
    private DbtCommand _currentCommand;

    // Replace with appropriate types and constructor parameters
    public DbtClient(IPythonEnvironment pythonEnvironment, ChannelWriter<EventStreamResponse> writer,
        ILogger<DbtClient> logger)
    {
        _pythonEnvironment = pythonEnvironment;
        _writer = writer;
        _logger = logger;
    }

    public OneOf<Success<string>, Error<string>> InitProject(string projectPath, string dbtProfilesDir)
    {
        string projectKey = GetPythonVariableNameFromPath(projectPath);
        var result =
            EvalDbtCommand($"{projectKey} = DbtProject(project_dir={projectPath}, profiles_dir={dbtProfilesDir})");
        return result.Match<OneOf<Success<string>, Error<string>>>(
            _ =>
            {
                _products.Add(projectKey, (projectPath, dbtProfilesDir));
                return new Success<string>(projectKey);
            },
            error =>
            {
                _logger.LogError(error.Value, "Failed to initialize DbtProject");
                return new Error<string>(error.Value.Message);
            }
        );
    }

    public bool IsProjectInitialized(string projectPath) =>
        _products.Keys.Contains(GetPythonVariableNameFromPath(projectPath));

    public OneOf<string, None> GetProjectKey(string projectPath)
    {
        var key = GetPythonVariableNameFromPath(projectPath);
        return _products.ContainsKey(key) ? key : new None();
    }

    public async Task<OneOf<Success, Error<string>>> ExecuteDbtCommand(DbtCommand command, CancellationToken cancellationToken)
    {
        if (string.IsNullOrWhiteSpace(command.CommandAsString))
        {
            _logger.LogWarning("Received empty command");
            return new Error<string>("Command cannot be empty");
        }
        if (_isCommandRunning)
            return new Error<string>("Another command is already running");
        _isCommandRunning = true;

        try
        {
            _logger.LogInformation("{ProcessReferenceId} - Starting command: {Command}", command.ProcessReferenceId,
                command.CommandAsString);
            _currentCommand = command;
            _messages.Clear();
            _messages.Enqueue(command.StatusMessage);
            
            var pipe = PipeTarget.ToDelegate(WriteBuffered);
            var cmd = Cli.Wrap(command.CommandAsString)
                .WithWorkingDirectory(command.Cwd)
                .WithStandardErrorPipe(pipe)
                .WithStandardOutputPipe(pipe);
            var result = await cmd.ExecuteAsync(cancellationToken);
            while (_messages.Count > 0)
                await FlushAsync();
            _logger.LogInformation("{ProcessReferenceId} - Finished command: {Command}", command.ProcessReferenceId,
                command.CommandAsString);
            _logger.LogInformation("{ProcessReferenceId} - Command result: {Result}", command.ProcessReferenceId,
                result.ExitCode);
            _logger.LogInformation("{ProcessReferenceId} - Command execution time: {ExecutionTime}",
                command.ProcessReferenceId, result.RunTime);
        }
        catch (Exception e)
        {
            const string msg = "Failed to execute command";
            _logger.LogError(e, msg);
            return new Error<string>(msg);
        }
        finally
        {
            _isCommandRunning = false;
        }
        return new Success();
    }

    public OneOf<Success<OneOf<PyObject, None>>, Error<Exception>> EvalDbtCommand(string command)
    {
        var initialize = _pythonEnvironment.Initialize();
        if (initialize.IsT1)
            return initialize.AsT1;

        return string.IsNullOrWhiteSpace(command) ? 
            new Error<Exception>(new ArgumentException("Command cannot be empty")) : 
            _pythonEnvironment.Eval(command);
    }

    private string GetPythonVariableNameFromPath(string projectPath) =>
        NormalizeVariableName(Path.GetFileName(Path.GetDirectoryName(projectPath)));

    private string NormalizeVariableName(string input)
    {
        var normalized = Regex.Replace(input, "[^a-zA-Z0-9]+", "_").Trim('_');
        if (Char.IsDigit(normalized[0]))
            normalized = "_" + normalized;

        return normalized;
    }

    private async Task WriteBuffered(string line)
    {
        _messages.Enqueue(line);
        if (_messages.Count > MessageBatchSize)
            await FlushAsync();
    }

    private async Task FlushAsync()
    {
        int size = _messages.Count > MessageBatchSize ? MessageBatchSize : _messages.Count;
        string[] lines = new string[size];
        for (int i = 0; i < size; i++)
            lines[i] = _messages.Dequeue();
        await _writer.WriteAsync(new EventStreamDbtLogResponse(_currentCommand.ProcessReferenceId, _currentCommand.CommandAsString, lines));
    }
}