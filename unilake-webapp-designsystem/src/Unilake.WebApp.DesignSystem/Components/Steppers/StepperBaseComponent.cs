using Microsoft.AspNetCore.Components;
namespace Unilake.WebApp.DesignSystem.Components.Steppers;

public abstract class StepperBaseComponent : UnilakeBaseComponent
{

    protected int Activestep;
    [EditorRequired] [Parameter] public required StepperItem[] Steps { get; set; }

    [Parameter]
    public int ActiveStep
    {
        get => Math.Min(Activestep, Steps.Length - 1);
        set => Activestep = Math.Min(Math.Abs(value), Steps.Length - 1);
    }

    protected bool IsLastStep(int step) => step + 1 == Steps.Length;
    protected bool IsActiveStep(int step) => step == ActiveStep;
    protected bool IsRegularStep(int step) => !IsLastStep(step) && !IsActiveStep(step);

}