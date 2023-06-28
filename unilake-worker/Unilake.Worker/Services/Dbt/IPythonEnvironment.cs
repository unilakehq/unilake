using OneOf;
using OneOf.Types;
using Python.Runtime;

namespace Unilake.Worker.Services.Dbt;

public interface IPythonEnvironment
{
    OneOf<Success, Error<Exception>> Initialize();
    OneOf<Success<OneOf<PyObject, None>>, Error<Exception>> Eval(string command);
}