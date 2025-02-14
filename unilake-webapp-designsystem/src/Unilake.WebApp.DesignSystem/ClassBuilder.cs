// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/ClassBuilder.cs

namespace Unilake.WebApp.DesignSystem;

using System;
using System.Collections.Generic;
using System.Linq;

public class ClassBuilder
{
    private readonly List<string> _classNames = new();

    public ClassBuilder(string? classNames = null) => Add(classNames);

    public ClassBuilder Add(string? className)
    {
        if (string.IsNullOrWhiteSpace(className)) return this;

        var classNames = className
            .Split([' '], StringSplitOptions.RemoveEmptyEntries)
            .Distinct()
            .ToList();

        foreach (var name in classNames.Where(name =>
                     !string.IsNullOrWhiteSpace(name) && !_classNames.Contains(name)))
            _classNames.Add(name);

        return this;
    }

    public ClassBuilder AddIf(string className, bool isOk) => isOk ? Add(className) : this;
    public ClassBuilder AddIfElse(string className, bool isOk, string falseClassName) => isOk ? Add(className) : Add(falseClassName);

    public ClassBuilder AddCompare<T>(T compare, Dictionary<T, string> with) where T : notnull
    {
        foreach (var kvp in with) AddCompare(kvp.Value, compare, kvp.Key);
        return this;
    }

    public ClassBuilder AddCompare<T>(string className, T compare, T with) =>
        AddIf(className, compare != null && compare.Equals(with));

    public ClassBuilder Remove(string className)
    {
        _classNames.RemoveAll(c => c.Equals(className, StringComparison.InvariantCultureIgnoreCase));
        return this;
    }

    public override string ToString()
    {
        if (_classNames.Count == 0)
            return string.Empty;

        return string.Join(" ", _classNames
            .Distinct()
            .Where(c => !string.IsNullOrWhiteSpace(c)));
    }
}