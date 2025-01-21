window.unilake = {
    copyToClipboard: (text) => {
        navigator.clipboard.writeText(text)
    },

    readFromClipboard: () => {
        return navigator.clipboard.readText();
    },

    setPropByElement: (element, property, value) => {
        element[property] = value;
        return "";
    },

    disableDraggable: (container, element) => {
        element.addEventListener("mousedown",
            (e) => {
                e.stopPropagation();
                container['draggable'] = false;
            });

        element.addEventListener("mouseup",
            (e) => {
                container['draggable'] = true;
            });

        element.addEventListener("mouseleave",
            (e) => {
                container['draggable'] = true;
            });
    },

    clickOutsideHandler: {
        removeEvent: (elementId) => {
            if (elementId === undefined || window.clickHandlers === undefined) return;
            if (!window.clickHandlers.has(elementId)) return;

            var handler = window.clickHandlers.get(elementId);
            window.removeEventListener("click", handler);
            window.clickHandlers.delete(elementId);
        },
        addEvent: (elementId, unregisterAfterClick, dotnetHelper) => {
            window.unilake.clickOutsideHandler.removeEvent(elementId);

            if (window.clickHandlers === undefined) {
                window.clickHandlers = new Map();
            }
            var currentTime = (new Date()).getTime();

            var handler = (e) => {

                var nowTime = (new Date()).getTime();
                var diff = Math.abs((nowTime - currentTime) / 1000);

                if (diff < 0.1)
                    return;

                currentTime = nowTime;

                var element = document.getElementById(elementId);
                if (e != null && element != null) {
                    if (e.target.isConnected === true && e.target !== element && (!element.contains(e.target))) {
                        if (unregisterAfterClick) {
                            window.unilake.clickOutsideHandler.removeEvent(elementId);
                        }
                        dotnetHelper.invokeMethodAsync("InvokeClickOutside");
                    }
                }
            };
            window.clickHandlers.set(elementId, handler);
            window.addEventListener("click", handler);
        }
    }
}