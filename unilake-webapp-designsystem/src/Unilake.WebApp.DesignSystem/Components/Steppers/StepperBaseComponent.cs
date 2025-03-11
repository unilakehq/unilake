using Microsoft.AspNetCore.Components;
namespace Unilake.WebApp.DesignSystem.Components.Steppers;

public abstract class StepperBaseComponent : UnilakeBaseComponent
{
    [Parameter, EditorRequired] public required StepperItem[] Steps { get; set; }

    [Parameter]
    public int ActiveStep
    {
        get => Math.Min(_activestep, Steps.Length - 1);
        set => _activestep = Math.Min(Math.Abs(value), Steps.Length - 1);
    }

    private int _activestep;
    protected bool IsLastStep(int step) => step + 1 == Steps.Length;
    protected bool IsActiveStep(int step) => step == ActiveStep;
    protected bool IsRegularStep(int step) => !IsLastStep(step) && !IsActiveStep(step);

}