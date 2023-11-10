using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Unilake.Www.Pages.Posts;

public class TaggedModel : PageModel
{
    [FromRoute]
    public string? Slug { get; set; }
}