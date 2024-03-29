// Copyright 2016-2022, Pulumi Corporation

using Pulumi;

namespace Unilake.Iac.Kubernetes.Custom.Crds
{
    /// <summary>
    /// A base class for all Kubernetes resources.
    /// </summary>
    public abstract class KubernetesResource : CustomResource
    {
        /// <summary>
        /// Standard constructor passing arguments to <see cref="CustomResource"/>.
        /// </summary>
        protected KubernetesResource(string type, string name, ResourceArgs? args, CustomResourceOptions? options = null)
            : base(type, name, args, options)
        {
        }

        /// <summary>
        /// Additional constructor for dynamic arguments received from YAML-based sources.
        /// </summary>
        protected KubernetesResource(string type, string name, DictionaryResourceArgs? args, CustomResourceOptions? options = null)
            : base(type, name, args, options)
        {
        }
    }
}