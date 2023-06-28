using OneOf;
using OneOf.Types;
using Python.Runtime;

namespace Unilake.Worker.Services.Dbt;

public sealed class PythonEnvironment : IPythonEnvironment, IDisposable
{
        private bool _isInitialized;
        private bool _isRunning;
        
        private PythonEnvironment()
        {
            // TODO: properly get this from config
            string pyDllPath = "C:\\Python38\\python38.dll"; // change this to the path of your Python DLL
            Environment.SetEnvironmentVariable("PYDLL", pyDllPath);
        }

        public OneOf<Success, Error<Exception>> Initialize()
        {
            if (_isInitialized) return new Success();
            try
            {
                PythonEngine.Initialize();
                _isInitialized = true;

                using (Py.GIL())
                {
                    PythonEngine.Exec(
                        System.IO.File.ReadAllText(Path.Join(Environment.CurrentDirectory, "dbt_integration.py")));
                }
            }
            catch (Exception e)
            {
                return new Error<Exception>(e);
            }
            return new Success();
        }

        public OneOf<Success<OneOf<PyObject, None>>, Error<Exception>> Eval(string command)
        {
            try
            {
                if (!_isInitialized)
                    throw new InvalidOperationException("Python interpreter is not initialized");
                if (_isRunning) return new Error<Exception>(new Exception("Python interpreter is already running"));
                _isRunning = true;

                using (Py.GIL())
                {
                    PyObject result = PythonEngine.Eval(command);
                    return result.IsNone()
                        ? new Success<OneOf<PyObject, None>>(new None())
                        : new Success<OneOf<PyObject, None>>(result);
                }
            }
            catch (Exception e)
            {
                return new Error<Exception>(e);
            }
            finally
            {
                _isRunning = false;
            }
        }
        
        public void Dispose()
        {
            if (!_isInitialized) return;
            PythonEngine.Shutdown();
            _isInitialized = false;
        }
    }