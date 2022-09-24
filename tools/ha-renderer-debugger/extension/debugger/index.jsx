const { useState, useEffect, useRef, useLayoutEffect, useReducer } = React;

Object.defineProperty(Array.prototype, 'findMap', {
    value: function (callback) {
        for (let index = 0; index < this.length; ++index) {
            const item = callback(this[index], index, this);
            if (item !== undefined) {
                return item;
            }
        }
    },
});

Object.defineProperty(Array.prototype, 'filterMap', {
    value: function (callback) {
        const result = [];
        for (let index = 0; index < this.length; ++index) {
            const item = callback(this[index], index, this);
            if (item !== undefined) {
                result.push(item);
            }
        }
        return result;
    },
});

function emptyIterator() {
    return [][Symbol.iterator]();
}

function makeEnum(...args) {
    let result = {};
    for (const arg of args) {
        result[arg] = Symbol(arg);
    }
    return Object.freeze(result);
}

const isLittleEndian = (() => {
    const buffer = new ArrayBuffer(2);
    new DataView(buffer).setInt16(0, 255, true);
    return new Int16Array(buffer)[0] == 255;
})();

const ImageBufferFormat = makeEnum(
    'Unsupported',
    'Color',
    'FloatColor',
    'RGBA',
    'RGB',
    'Luminance',
    'Data',
);

const FilterAction = makeEnum(
    'Enable',
    'Add',
    'BulkAdd',
    'Remove',
    'Clear',
);

const FilterType = makeEnum(
    'RenderTarget',
    'Mesh',
    'Image',
    'Material',
);

const MatchAny = Symbol('MatchAny');

function matchAny(...args) {
    return value => args.some(arg => {
        return value === (typeof arg === 'function' ? arg(value) : arg);
    });
}

function matchAll(...args) {
    return value => args.every(arg => {
        return value === (typeof arg === 'function' ? arg(value) : arg);
    });
}

function doesMatchPattern(object, pattern, exact = false) {
    if (pattern === MatchAny) {
        return true;
    }
    else if (typeof pattern === 'function') {
        if (pattern(object)) {
            return true;
        }
        return false;
    }
    else if (pattern !== null && typeof pattern === 'object' && typeof object === 'object') {
        const patternKeys = Object.keys(pattern);
        if (exact) {
            const objectKeys = Object.keys(object);
            if (patternKeys.length !== objectKeys.length) {
                return false;
            }
            for (let index = 0; index <= patternKeys.length; ++index) {
                if (patternKeys[index] !== objectKeys[index]) {
                    return false;
                }
            }
        }
        return patternKeys.every(key => {
            if (!object.hasOwnProperty(key)) {
                return false;
            }
            if (pattern[key] === MatchAny) {
                return true;
            }
            return doesMatchPattern(object[key], pattern[key], exact);
        });
    }
    else if (pattern === object) {
        return true;
    }
    return false;
}

function match(object, ...args) {
    for (let i = 0; i + 1 < args.length; i += 2) {
        const pattern = args[i];
        const result = args[i + 1];
        if (doesMatchPattern(object, pattern)) {
            return typeof result === 'function' ? result(object) : result;
        }
    }
}

class BridgeEvent extends Event {
    constructor(type, data, binary, options) {
        super(type, options);
        this.timestamp = new Date().toLocaleString();
        this.data = data;
        this.binary = binary;
    }
}

class Bridge extends EventTarget {
    constructor(name, version = 0, timeout = 3000) {
        super();
        this._snapshot = null;
        this._version = version | 0;
        this._timeout = timeout | 3000;
        this._channel = new BroadcastChannel(name);
        this._channel.addEventListener('message', msg => {
            const { id, version, text, binary } = msg.data;
            if (version === this._version) {
                console.log('Bridge received:', id);
                this.dispatchEvent(new BridgeEvent(id, !!text ? JSON.parse(text) : undefined, binary));
            }
        });
    }

    dispose() {
        this._snapshot = null;
        if (!!this._channel) {
            this._channel.removeEventListener('message');
            this._channel.close();
            this._channel = null;
        }
    }

    get snapshot() {
        return this._snapshot;
    }

    set snapshot(value) {
        this._snapshot = value;
        this.dispatchEvent(new BridgeEvent(
            'SnapshotChanged',
            !!value ? value.data : undefined,
            !!value ? value.binary : undefined,
        ));
    }

    send(id, data, binary) {
        this._channel.postMessage({
            id,
            version: this._version,
            text: !!data ? JSON.stringify(data) : undefined,
            binary,
        });
        console.log('Bridge sent:', id);
    }

    receive(id, validation) {
        const self = this;
        return new Promise(resolve => {
            const context = {};
            context.callback = event => {
                if (typeof validation !== 'function' || validation(event)) {
                    self.removeEventListener(id, context.callback);
                    resolve(event);
                }
            };
            self.addEventListener(id, context.callback);
            setTimeout(
                () => self.removeEventListener(id, context.callback),
                this._timeout,
            );
        });
    }

    provide(id, data, binary) {
        console.log('Bridge provided:', id);
        this.dispatchEvent(new BridgeEvent(id, data, binary));
    }

    checkPulse() {
        const type = 'CheckPulse';
        const result = this.receive(type);
        this.send(type);
        return result;
    }

    takeSnapshot() {
        const type = 'TakeSnapshot';
        const result = this.receive(type);
        this.send(type);
        return result;
    }

    listStages() {
        const type = 'ListStages';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.stages);
        } else {
            this.send(type);
        }
        return result;
    }

    listPipelines() {
        const type = 'ListPipelines';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.pipelines.map(item => item.id));
        } else {
            this.send(type);
        }
        return result;
    }

    listRenderTargets() {
        const type = 'ListRenderTargets';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.render_targets.map(item => item.id));
        } else {
            this.send(type);
        }
        return result;
    }

    listMeshes() {
        const type = 'ListMeshes';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.meshes.map(item => item.id));
        } else {
            this.send(type);
        }
        return result;
    }

    listImages() {
        const type = 'ListImages';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.images.map(item => item.id));
        } else {
            this.send(type);
        }
        return result;
    }

    listMaterials() {
        const type = 'ListMaterials';
        const result = this.receive(type);
        if (!!bridge.snapshot) {
            this.provide(type, bridge.snapshot.data.materials.map(item => item.id));
        } else {
            this.send(type);
        }
        return result;
    }

    queryPipeline(id) {
        const type = 'QueryPipeline';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.pipelines.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryPipelineResources(id) {
        const type = 'QueryPipelineResources';
        const result = this.receive(type, event => event.data.id === id);
        this.send(type, id);
        return result;
    }

    queryPipelineStageRenderQueue(id, index) {
        const type = 'QueryPipelineStageRenderQueue';
        const result = this.receive(type, event => event.data.id === id && event.data.stage_index === index);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.pipelines_render_queues
                .find(item => item.id === id && item.stage_index == index);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, { id, stage_index: index });
        }
        return result;
    }

    queryPipelineStageRenderQueueResources(id, index) {
        const type = 'QueryPipelineStageRenderQueueResources';
        const result = this.receive(type, event => event.data.id === id && event.data.stage_index === index);
        this.send(type, { id, stage_index: index });
        return result;
    }

    queryRenderTarget(id) {
        const type = 'QueryRenderTarget';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.render_targets.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryRenderTargetColorData(id, index) {
        const type = 'QueryRenderTargetColorData';
        const result = this.receive(type, event => event.data.id === id && event.data.attachment_index === index);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.render_targets_color_data
                .find(item => item.id === id && item.attachment_index === index);
            if (!!found) {
                this.provide(type, found, bridge.snapshot.binary);
            }
        } else {
            this.send(type, { id, attachment_index: index });
        }
        return result;
    }

    queryMesh(id) {
        const type = 'QueryMesh';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.meshes.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryMeshVertexData(id, index) {
        const type = 'QueryMeshVertexData';
        const result = this.receive(type, event => event.data.id === id && event.data.buffer_index === index);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.meshes_data.find(item => item.id === id);
            if (!!found) {
                const data = {
                    id: found.id,
                    buffer_index: index,
                    layout: found.layout.buffers[index],
                    bytes_range: found.vertex_bytes_ranges[index],
                };
                this.provide(type, data, bridge.snapshot.binary);
            }
        } else {
            this.send(type, { id, buffer_index: index });
        }
        return result;
    }

    queryMeshIndexData(id) {
        const type = 'QueryMeshIndexData';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.meshes_data.find(item => item.id === id);
            if (!!found) {
                const data = {
                    id: found.id,
                    draw_mode: found.draw_mode,
                    bytes_range: found.index_bytes_range,
                };
                this.provide(type, data, bridge.snapshot.binary);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryMeshData(id) {
        const type = 'QueryMeshData';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.meshes_data.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found, bridge.snapshot.binary);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryImage(id) {
        const type = 'QueryImage';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.images.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryImageData(id) {
        const type = 'QueryImageData';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.images_data.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found, bridge.snapshot.binary);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }

    queryMaterial(id) {
        const type = 'QueryMaterial';
        const result = this.receive(type, event => event.data.id === id);
        if (!!bridge.snapshot) {
            const found = bridge.snapshot.data.materials.find(item => item.id === id);
            if (!!found) {
                this.provide(type, found);
            }
        } else {
            this.send(type, id);
        }
        return result;
    }
}

const bridge = new Bridge(OxygengineHARD.channelName, 0);
const snapshots = [];
if (!!OxygengineHARD.startWithSnapshot) {
    setTimeout(() => {
        bridge.takeSnapshot().then(event => {
            bridge.snapshot = snapshots.find(item => item.timestamp === event.timestamp);
        });
    }, 0);
}

function useOpenClose(onOpen, onClose) {
    useEffect(() => {
        !!onOpen && onOpen();
        return () => !!onClose && onClose();
    }, []);
}

function useOpen(callback) {
    useOpenClose(callback, null);
}

function useClose(callback) {
    useOpenClose(null, callback);
}

function useBridge(name, callback) {
    useOpenClose(
        () => bridge.addEventListener(name, callback),
        () => bridge.removeEventListener(name, callback),
    );
}

function useDirty() {
    let [counter, setCounter] = useState(0);
    return () => setCounter(++counter);
}

const widgets = {};

function Button(props) {
    const styles = {
        fill: {
            width: 'calc(100% - 8px)',
            height: 'max-content',
            margin: '4px',
        },
    };

    return (
        <button
            style={{ ...(!props.compact ? styles.fill : null), ...props.style }}
            type="button"
            onClick={!!props.onClick ? () => props.onClick() : null}
        >
            {props.label}
            {props.children}
        </button>
    );
}
widgets.Button = Button;

function Toggle(props) {
    const styles = {
        container: {
            width: '100%',
            cursor: 'pointer',
        },
        toggle: {
            marginLeft: 0,
            paddingLeft: 0,
        },
        label: {
            color: !!props.labelAccent ? 'yellow' : undefined,
        },
    };

    return (
        <div
            style={{ ...styles.container, ...props.style }}
            onClick={!!props.onChange ? () => props.onChange(!props.on) : null}
        >
            <input
                style={styles.toggle}
                type="checkbox"
                checked={props.on}
                readOnly
            />
            <span style={styles.label}>{props.label}</span>
            {props.children}
        </div>
    );
}
widgets.Toggle = Toggle;

function Field(props) {
    const styles = {
        content: {
            marginLeft: '6px',
        },
        label: {
            color: !!props.accent ? 'yellow' : 'white',
            cursor: 'pointer',
        },
    };

    if (!!props.novalue) {
        return (
            <div style={props.style}>
                <span style={styles.label}>{props.label}</span>
            </div>
        );
    } else if (!!props.inline) {
        return (
            <div style={props.style}>
                <span style={styles.label}>{props.label}</span>: <span>{props.children}</span>
            </div>
        );
    } else {
        return (
            <details style={props.style} open={!!props.open}>
                <summary style={styles.label}>{props.label}</summary>
                <div style={styles.content}>
                    {props.children}
                </div>
            </details>
        );
    }
}
widgets.Field = Field;

function ObjectField(props) {
    const key = props.label;
    const value = props.object;
    const { open, contentonly } = props;

    return match(
        value,
        undefined,
        () => (<Field style={props.style} label={key} open={open} inline>undefined</Field>),
        null,
        () => (<Field style={props.style} label={key} open={open} inline>null</Field>),
        () => typeof value === 'symbol',
        () => (<Field style={props.style} label={key} open={open} inline>{value.toString()}</Field>),
        () => typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean',
        () => (<Field style={props.style} label={key} open={open} inline>{`${value}`}</Field>),
        () => Array.isArray(value),
        () => {
            const children = value.map((value, index) => (
                <widgets.ObjectField
                    key={index}
                    label={index}
                    object={value}
                    open={open}
                />
            ));
            return contentonly ? children : (
                <Field style={props.style} label={`${key} [${value.length}]`} open={open}>
                    {children}
                </Field>
            );
        },
        () => typeof value === 'object',
        () => {
            const children = Object.entries(value).map(([key, value]) => (
                <widgets.ObjectField
                    key={key}
                    label={key}
                    object={value}
                    open={open}
                />
            ));
            return contentonly ? children : (
                <Field style={props.style} label={`${key} {${Object.keys(value).length}}`} open={open}>
                    {children}
                </Field>
            );
        },
        MatchAny,
        null,
    );
}
widgets.ObjectField = ObjectField;

function Tabs(props) {
    const styles = {
        container: {
            width: '100%',
            height: 'min-content',
        },
        tabs: {
            width: '100%',
            height: 'min-content',
            textAlign: 'center',
        },
        activeTab: {
            fontWeight: 'bold',
            fontStyle: 'italic',
        },
        content: {
            width: '100%',
            height: 'min-content',
        },
    };

    const children = !!props.children
        ? (Array.isArray(props.children) ? props.children : [props.children])
        : [];

    const tabs = children
        .filter(child => !!child.key)
        .map(child => (
            <Button
                key={child.key}
                label={child.props.tabLabel || child.key}
                style={props.activeTab === child.key ? styles.activeTab : null}
                onClick={!!props.onChange ? () => props.onChange(child.key) : null}
                compact
            />
        ));

    const selectedTab = !!props.activeTab
        ? children.find(child => child.key === props.activeTab) || null
        : null;

    return (
        <div style={{ ...styles.container, ...props.style }}>
            <div style={styles.tabs}>
                {tabs}
            </div>
            <div style={styles.content}>
                {selectedTab}
            </div>
        </div>
    );
}
widgets.Tabs = Tabs;

function Tab(props) {
    const styles = {
        container: {
            width: '100%',
        },
    };

    useOpenClose(
        () => !!props.onShow && props.onShow(),
        () => !!props.onHide && props.onHide(),
    );

    return (
        <div style={{ ...styles.container, ...props.style }}>
            {props.children}
        </div>
    );
}
widgets.Tab = Tab;

function Conditional(props) {
    const show = typeof props.condition === 'function' ? props.condition() : !!props.condition;
    return show ? props.children : null;
}
widgets.Conditional = Conditional;

function Section(props) {
    const styles = {
        container: {
            borderTop: 'solid 1px gray',
            borderBottom: 'solid 1px gray',
        },
        content: {
            marginLeft: '10px',
            marginTop: '5px',
            marginBottom: '5px',
        },
    };

    return (
        <div style={{ ...styles.container, ...props.style }}>
            <Toggle
                label={props.label}
                labelAccent={props.labelAccent}
                on={!!props.open}
                onChange={props.onChange}
            />
            {!!props.open ? (<div style={styles.content}>{props.children}</div>) : null}
        </div>
    );
}
widgets.Section = Section;

function AutoSection(props) {
    const [open, setOpen] = useState(!!props.open);

    return (
        <Section
            style={props.style}
            label={props.label}
            labelAccent={props.labelAccent}
            open={open}
            onChange={checked => setOpen(checked)}
        >
            {props.children}
        </Section>
    );
}
widgets.AutoSection = AutoSection;

function ImageBufferPixelPreview(props) {
    const { width, height, depth, format, bytes } = props;
    const canvasRef = useRef(null);
    const canvasParentRef = useRef(null);
    const [flipHor, setFlipHor] = useState(!!props.flipHor);
    const [flipVer, setFlipVer] = useState(!!props.flipVer);
    const styles = {
        canvas: {
            width: '100%',
            height: '100%',
            objectFit: 'contain',
            transform: `scale(${flipHor ? -1 : 1}, ${flipVer ? -1 : 1})`,
        },
        canvasParent: {
            width: '100%',
            aspectRatio: `${width} / ${height}`,
            backgroundImage: `url(${OxygengineHARD.images.checkerboard})`,
        },
    };

    useOpen(() => {
        if (!!canvasRef.current) {
            const context = canvasRef.current.getContext('2d');
            if (!!context) {
                const data = match(
                    format,
                    matchAny(ImageBufferFormat.Color, ImageBufferFormat.RGBA),
                    () => new ImageData(bytes, width, height),
                    ImageBufferFormat.RGB,
                    () => {
                        const data = new Uint8ClampedArray(width * height * 4);
                        for (let index = 0; index < width * height; ++index) {
                            const source = index * 3;
                            const target = index * 4;
                            data[target] = bytes[source];
                            data[target + 1] = bytes[source + 1];
                            data[target + 2] = bytes[source + 2];
                            data[target + 3] = 255;
                        }
                        return new ImageData(data, width, height);
                    },
                    ImageBufferFormat.Luminance,
                    () => {
                        const data = new Uint8ClampedArray(width * height * 4);
                        for (let index = 0; index < width * height; ++index) {
                            const value = bytes[index];
                            const target = index * 4;
                            data[target] = value;
                            data[target + 1] = value;
                            data[target + 2] = value;
                            data[target + 3] = 255;
                        }
                        return new ImageData(data, width, height);
                    },
                    MatchAny,
                    () => new ImageData(width, height),
                );
                context.putImageData(data, 0, 0);
            }
        }
    });

    return (
        <div style={props.style}>
            <Toggle
                label="Flip horizontally"
                on={flipHor}
                onChange={checked => setFlipHor(checked)}
            />
            <Toggle
                label="Flip vertically"
                on={flipVer}
                onChange={checked => setFlipVer(checked)}
            />
            <Button
                label="Show fullscreen"
                onClick={() => {
                    if (!!canvasParentRef.current) {
                        canvasParentRef.current.requestFullscreen();
                    }
                }}
            />
            <div ref={canvasParentRef} style={styles.canvasParent}>
                <canvas
                    ref={canvasRef}
                    width={width}
                    height={height}
                    style={styles.canvas}
                />
            </div>
        </div>
    );
}
widgets.ImageBufferPixelPreview = ImageBufferPixelPreview;

function ImageBufferDataPreview(props) {
    const { width, height, depth, format, bytes } = props;
    const styles = {
        accent: {
            color: 'white',
        },
        cell: {
            border: '1px solid orange',
            padding: 6,
        },
        center: {
            textAlign: 'center',
        },
    };
    const [view, channels] = match(
        format,
        matchAny(ImageBufferFormat.FloatColor, ImageBufferFormat.Data),
        () => [new Float32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 4), 4],
        matchAny(ImageBufferFormat.Color, ImageBufferFormat.RGBA),
        () => [bytes, 4],
        ImageBufferFormat.RGB,
        () => [bytes, 3],
        ImageBufferFormat.Luminance,
        () => [bytes, 1],
        MatchAny,
        null,
    );

    const children = [];
    if (!!view) {
        for (let index = 0; index < view.length / channels; ++index) {
            const x = (index % width) | 0;
            const y = (index / width) | 0;
            const z = (index / (width * height)) | 0;
            const offset = index * channels;
            children.push(
                <tr key={index}>
                    <td style={{ ...styles.cell, ...styles.accent }}>{`${x}`}</td>
                    <td style={{ ...styles.cell, ...styles.accent }}>{`${y}`}</td>
                    <td style={{ ...styles.cell, ...styles.accent }}>{`${z}`}</td>
                    <td style={styles.cell}>{`${view[offset]}`}</td>
                    {channels >= 2 ? (<td style={styles.cell}>{`${view[offset + 1]}`}</td>) : null}
                    {channels >= 3 ? (<td style={styles.cell}>{`${view[offset + 2]}`}</td>) : null}
                    {channels >= 4 ? (<td style={styles.cell}>{`${view[offset + 3]}`}</td>) : null}
                </tr>
            );
        }
    }

    return (
        <table style={props.style}>
            <thead>
                <tr>
                    <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>#X</th>
                    <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>#Y</th>
                    <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>#Z</th>
                    <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>Red</th>
                    {channels >= 2 ? (
                        <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>Green</th>
                    ) : null}
                    {channels >= 3 ? (
                        <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>Blue</th>
                    ) : null}
                    {channels >= 4 ? (
                        <th style={{ ...styles.cell, ...styles.accent, ...styles.center }}>Alpha</th>
                    ) : null}
                </tr>
            </thead>
            <tbody>
                {children}
            </tbody>
        </table>
    );
}
widgets.ImageBufferDataPreview = ImageBufferDataPreview;

function ImageBufferPreview(props) {
    const { width, height, depth, format, bytes } = props;
    const [showData, setShowData] = useState(false);

    const formatName = match(
        format,
        ImageBufferFormat.Unsupported,
        'Unsupported',
        ImageBufferFormat.Color,
        'Color',
        ImageBufferFormat.FloatColor,
        'FloatColor',
        ImageBufferFormat.RGBA,
        'RGBA',
        ImageBufferFormat.RGB,
        'RGB',
        ImageBufferFormat.Luminance,
        'Luminance',
        ImageBufferFormat.Data,
        'Data',
    );

    const children = !!showData ? (
        <ImageBufferDataPreview
            width={width}
            height={height}
            depth={depth}
            format={format}
            bytes={bytes}
        />
    ) : (
        <ImageBufferPixelPreview
            width={width}
            height={height}
            depth={depth}
            format={format}
            bytes={bytes}
        />
    );

    return (
        <div style={props.style}>
            <Field label="Format" inline>{formatName}</Field>
            <Field label="Width" inline>{width}</Field>
            <Field label="Height" inline>{height}</Field>
            <Field label="Depth" inline>{depth}</Field>
            <Toggle
                label="Show as data"
                on={showData}
                onChange={checked => setShowData(checked)}
            />
            {children}
        </div>
    );
}
widgets.ImageBufferPreview = ImageBufferPreview;

function Filter(props) {
    const { label, query, type, filters, dispatchFilters } = props;
    const source = match(
        type,
        FilterType.RenderTarget,
        filters.renderTargets,
        FilterType.Mesh,
        filters.meshes,
        FilterType.Image,
        filters.images,
        FilterType.Material,
        filters.materials,
        MatchAny,
        null,
    );
    const included = !!source && source.some(item => doesMatchPattern(item, query));
    const children = !label
        ? (<ObjectField label="Query" object={query} open />)
        : null;

    return (
        <Toggle
            label={label}
            noformat={!label}
            onChange={checked => dispatchFilters({
                action: checked ? FilterAction.Add : FilterAction.Remove,
                type,
                query,
            })}
            on={included}
        >
            {children}
        </Toggle>
    );
}
widgets.Filter = Filter;

function Stage(props) {
    return (
        <Field style={props.style} label={props.name} open>
            <span>{props.type}</span>
        </Field>
    );
}
widgets.Stage = Stage;

function StagesTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info.map((item, index) => (
            <Stage key={index} name={item.stage_name} type={item.type_name} />
        ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listStages().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.StagesTab = StagesTab;

function PipelineRenderTarget(props) {
    return (
        <Field style={props.style} label={props.name} open>
            <Filter
                label="Use in filters"
                query={props.id}
                type={FilterType.RenderTarget}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            <Field label="ID" inline>{props.id}</Field>
            <ObjectField label="Descriptor" object={props.descriptor} />
        </Field>
    );
}
widgets.PipelineRenderTarget = PipelineRenderTarget;

function PipelineStageRenderQueueTableCommand(props) {
    const { command } = props;

    return match(
        command,
        'SortingBarrier',
        () => (<Field label="Sorting barrier" novalue accent />),
        { 'Viewport': MatchAny },
        () => (
            <Field label="Viewport" accent>
                <Field label="X" inline>{command.Viewport[0]}</Field>
                <Field label="Y" inline>{command.Viewport[1]}</Field>
                <Field label="Width" inline>{command.Viewport[2]}</Field>
                <Field label="Height" inline>{command.Viewport[3]}</Field>
            </Field>
        ),
        { 'ActivateMaterial': MatchAny },
        () => (
            <Field label="Activate material" accent>
                <Field label="ID" inline>{command.ActivateMaterial[0]}</Field>
                <ObjectField label="Material signature" object={command.ActivateMaterial[1]} open />
            </Field>
        ),
        { 'OverrideUniform': MatchAny },
        () => (
            <Field label="Override uniform" accent>
                <Field label="Name" inline>{command.OverrideUniform[0]}</Field>
                <ObjectField label="Material value" object={command.OverrideUniform[1]} open />
            </Field>
        ),
        { 'ResetUniform': MatchAny },
        () => (
            <Field label="Reset uniform" accent>
                <Field label="Name" inline>{command.ResetUniform}</Field>
            </Field>
        ),
        'ResetUniforms',
        () => (<Field label="Reset uniforms" novalue accent />),
        { 'ApplyDrawOptions': MatchAny },
        () => (
            <Field label="Apply draw options" accent>
                <ObjectField label="Material draw options" object={command.ApplyDrawOptions} open />
            </Field>
        ),
        { 'ActivateMesh': MatchAny },
        () => (
            <Field label="Activate mesh" accent>
                <Field label="ID" inline>{command.ActivateMesh}</Field>
            </Field>
        ),
        { 'DrawMesh': MatchAny },
        () => (
            <Field label="Draw mesh" accent>
                <ObjectField label="Mesh draw range" object={command.DrawMesh} open />
            </Field>
        ),
        { 'Scissor': null },
        () => (<Field label="Scissor" inline accent>None</Field>),
        { 'Scissor': MatchAny },
        () => (
            <Field label="Scissor" accent>
                <Field label="X" inline>{command.Scissor[0]}</Field>
                <Field label="Y" inline>{command.Scissor[1]}</Field>
                <Field label="Width" inline>{command.Scissor[2]}</Field>
                <Field label="Height" inline>{command.Scissor[3]}</Field>
            </Field>
        ),
        MatchAny,
        null,
    );
}
widgets.PipelineStageRenderQueueTableCommand = PipelineStageRenderQueueTableCommand;

function PipelineStageRenderQueueTableCommands(props) {
    const styles = {
        cell: {
            border: '1px solid orange',
            padding: 3,
        },
        center: {
            textAlign: 'center',
        },
    };
    const children = props.chunks
        .map(({ group, commands }, groupIndex) => commands.map(({ order, command }, orderIndex) => (
            <tr key={`${groupIndex}-${orderIndex}`}>
                {orderIndex === 0 ? (
                    <td
                        style={{ ...styles.cell, ...styles.center }}
                        rowSpan={commands.length}>
                        {`${group}`}
                    </td>
                ) : null}
                <td style={{ ...styles.cell, ...styles.center }}>{`${order}`}</td>
                <td style={styles.cell}>
                    <PipelineStageRenderQueueTableCommand command={command} />
                </td>
            </tr>
        )))
        .flat();

    return (
        <table>
            <thead>
                <tr>
                    <th style={{ ...styles.cell, ...styles.center }}>Group</th>
                    <th style={{ ...styles.cell, ...styles.center }}>Order</th>
                    <th style={styles.cell}>Command</th>
                </tr>
            </thead>
            <tbody>
                {children}
            </tbody>
        </table>
    );
}
widgets.PipelineStageRenderQueueTableCommands = PipelineStageRenderQueueTableCommands;

function PipelineStageRenderQueuePrettyCommand(props) {
    const {
        label,
        viewport,
        material,
        uniforms,
        uniformsReset,
        drawOptions,
        mesh,
        meshRange,
        scissor,
    } = props;
    const styles = {
        accent: {
            color: 'yellow',
        },
        cell: {
            border: '1px solid orange',
            padding: 3,
        },
        center: {
            textAlign: 'center',
        },
    };

    const meshChildren = !!mesh && !!meshRange ? (
        <fieldset>
            <legend>Mesh</legend>
            <Field label="ID" inline>{mesh}</Field>
            <ObjectField label="Draw range" object={meshRange} open />
        </fieldset>
    ) : null;

    const drawOptionsChildren = !!material && !!drawOptions ? (
        <ObjectField label="Draw options" object={drawOptions} open />
    ) : null;

    const uniformsChildren = !!material && !!uniforms && !!uniformsReset
        ? uniforms.map(({ name, value }, index) => (
            <tr key={index}>
                <td style={{ ...styles.cell, ...styles.accent, ...styles.center }}>
                    {name}
                </td>
                <td style={styles.cell}>
                    <ObjectField object={value} open contentonly />
                </td>
                <td style={styles.cell}>
                    <input type="checkbox" checked={uniformsReset.has(name)} readOnly />
                </td>
            </tr>
        ))
        : null;

    const materialChildren = !!material ? (
        <fieldset>
            <legend>Material</legend>
            <Field label="ID" inline>{material.id}</Field>
            <ObjectField label="Signature" object={material.signature} open />
            {drawOptionsChildren}
            <AutoSection label="Uniforms">
                <table>
                    <thead>
                        <tr>
                            <th style={{ ...styles.cell, ...styles.accent }}>Name</th>
                            <th style={{ ...styles.cell, ...styles.accent }}>Value</th>
                            <th style={{ ...styles.cell, ...styles.accent }}>Reset</th>
                        </tr>
                    </thead>
                    <tbody>
                        {uniformsChildren}
                    </tbody>
                </table>
            </AutoSection>
        </fieldset>
    ) : null;

    const viewportChildren = !!viewport ? (
        <Field label="Viewport" open>
            <Field label="X" inline>{viewport.x}</Field>
            <Field label="Y" inline>{viewport.y}</Field>
            <Field label="Width" inline>{viewport.w}</Field>
            <Field label="Height" inline>{viewport.h}</Field>
        </Field>
    ) : null;

    const scissorChildren = match(
        scissor,
        undefined,
        null,
        null,
        () => (<Field label="Scissor" inline>None</Field>),
        MatchAny,
        () => (
            <Field label="Scissor" open>
                <Field label="X" inline>{scissor.x}</Field>
                <Field label="Y" inline>{scissor.y}</Field>
                <Field label="Width" inline>{scissor.w}</Field>
                <Field label="Height" inline>{scissor.h}</Field>
            </Field>
        ),
    );

    return (
        <AutoSection label={label}>
            {meshChildren}
            {materialChildren}
            <fieldset>
                <legend>Miscellaneous</legend>
                {viewportChildren}
                {scissorChildren}
            </fieldset>
        </AutoSection>
    );
}
widgets.PipelineStageRenderQueuePrettyCommand = PipelineStageRenderQueuePrettyCommand;

function PipelineStageRenderQueuePrettyCommands(props) {
    const children = props.chunks.map((chunk, index) => {
        const viewport = chunk.commands.findMap(item => match(
            item.command,
            { 'Viewport': MatchAny },
            () => {
                const [x, y, w, h] = item.command.Viewport;
                return { x, y, w, h };
            },
        ));
        const material = chunk.commands.findMap(item => match(
            item.command,
            { 'ActivateMaterial': MatchAny },
            () => {
                const [id, signature] = item.command.ActivateMaterial;
                return { id, signature };
            },
        ));
        const uniforms = chunk.commands.filterMap(item => match(
            item.command,
            { 'OverrideUniform': MatchAny },
            () => {
                const [name, value] = item.command.OverrideUniform;
                return { name, value };
            },
        ));
        const uniformsReset = new Set();
        for (const item of chunk.commands) {
            match(
                item.command,
                'ResetUniforms',
                () => {
                    for (const item of uniforms) {
                        uniformsReset.add(item.name);
                    }
                },
                { 'ResetUniform': MatchAny },
                () => uniformsReset.add(item.command.ResetUniform),
            )
        }
        const drawOptions = chunk.commands.findMap(item => match(
            item.command,
            { 'ApplyDrawOptions': MatchAny },
            item.command.ApplyDrawOptions,
        ));
        const mesh = chunk.commands.findMap(item => match(
            item.command,
            { 'ActivateMesh': MatchAny },
            item.command.ActivateMesh,
        ));
        const meshRange = chunk.commands.findMap(item => match(
            item.command,
            { 'DrawMesh': MatchAny },
            item.command.DrawMesh,
        ));
        const scissor = chunk.commands.findMap(item => match(
            item.command,
            { 'Scissor': null },
            undefined,
            { 'Scissor': MatchAny },
            () => {
                const [x, y, w, h] = item.command.Scissor;
                return { x, y, w, h };
            },
        ));

        return (
            <PipelineStageRenderQueuePrettyCommand
                key={index}
                label={`Group: ${chunk.group}`}
                viewport={viewport}
                material={material}
                uniforms={uniforms}
                uniformsReset={uniformsReset}
                drawOptions={drawOptions}
                mesh={mesh}
                meshRange={meshRange}
                scissor={scissor}
            />
        );
    });

    return (
        <div style={props.style}>
            {children}
        </div>
    );
}
widgets.PipelineStageRenderQueuePrettyCommands = PipelineStageRenderQueuePrettyCommands;

function PipelineStageRenderQueue(props) {
    const [pretty, setPretty] = useState(true);
    const chunks = [];
    let lastGroup = -1;
    for (const item of props.data.commands) {
        const [[group, order], command] = item;
        if (group !== lastGroup) {
            chunks.push({ group, commands: [] });
        }
        lastGroup = group;
        chunks.at(-1).commands.push({ order, command });
    }

    const children = !!pretty
        ? (<PipelineStageRenderQueuePrettyCommands chunks={chunks} />)
        : (<PipelineStageRenderQueueTableCommands chunks={chunks} />);

    return (
        <div style={props.style}>
            <ObjectField label="Size" object={props.data.size} open />
            <Field label="Persistent" inline>{`${props.data.persistent}`}</Field>
            <Toggle
                label="Pretty commands view"
                on={pretty}
                onChange={mode => setPretty(mode)}
            />
            {children}
        </div>
    );
}
widgets.PipelineStageRenderQueue = PipelineStageRenderQueue;

function PipelineStage(props) {
    const [open, setOpen] = useState(false);
    const [renderQueue, setRenderQueue] = useState(null);

    return (
        <div style={props.style}>
            <ObjectField label={props.index} object={props.data} open />
            <Section
                label="Render Queue"
                open={open}
                onChange={checked => {
                    if (checked) {
                        bridge
                            .queryPipelineStageRenderQueue(props.pipeline, props.index)
                            .then(event => setRenderQueue(event.data.render_queue));
                    }
                    setOpen(checked);
                }}
            >
                <Button
                    label="Use in filters"
                    onClick={() => {
                        bridge
                            .queryPipelineStageRenderQueueResources(props.pipeline, props.index)
                            .then(event => {
                                props.dispatchFilters({
                                    action: FilterAction.BulkAdd,
                                    meshes: event.data.meshes,
                                    images: event.data.images,
                                    materials: event.data.materials.map(([id, signature]) => {
                                        return { id, signature };
                                    }),
                                });
                            });
                    }}
                />
                {!!renderQueue ? (<PipelineStageRenderQueue data={renderQueue} />) : null}
            </Section>
        </div>
    );
}
widgets.PipelineStage = PipelineStage;

function Pipeline(props) {
    const [open, setOpen] = useState(false);
    const [info, setInfo] = useState(null);

    const children = open && !!info ? [
        <Button
            key="use-in-filters"
            label="Use in filters"
            onClick={() => {
                bridge
                    .queryPipelineResources(props.id)
                    .then(event => {
                        props.dispatchFilters({
                            action: FilterAction.BulkAdd,
                            renderTargets: event.data.render_targets,
                            meshes: event.data.meshes,
                            images: event.data.images,
                            materials: event.data.materials.map(([id, signature]) => {
                                return { id, signature };
                            }),
                        });
                    });
            }}
        />,
        <fieldset key="render-targets">
            <legend>Render Targets</legend>
            {
                Object.entries(info.render_targets)
                    .map(([name, [descriptor, id]]) => (
                        <PipelineRenderTarget
                            key={name}
                            name={name}
                            descriptor={descriptor}
                            id={id}
                            filters={props.filters}
                            dispatchFilters={props.dispatchFilters}
                        />
                    ))
            }
        </fieldset>,
        <fieldset key="stages">
            <legend>Stages</legend>
            {
                info.stages.map((item, index) => (
                    <PipelineStage
                        key={index}
                        pipeline={props.id}
                        index={index}
                        data={item}
                        dispatchFilters={props.dispatchFilters}
                    />
                ))
            }
        </fieldset>,
    ] : null;

    return (
        <Section
            style={props.style}
            label={props.id}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryPipeline(props.id)
                        .then(event => setInfo(event.data.info));
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.Pipeline = Pipeline;

function PipelinesTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info.map((item, index) => (
            <Pipeline
                key={index}
                id={item}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
        ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listPipelines().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.PipelinesTab = PipelinesTab;

function RenderTargetPreview(props) {
    const [open, setOpen] = useState(false);
    const [preview, setPreview] = useState(null);

    const children = open && preview ? (
        <ImageBufferPreview
            width={preview.width}
            height={preview.height}
            depth={1}
            format={preview.format}
            flipVer
            bytes={preview.bytes}
        />
    ) : null;

    return (
        <Section
            style={props.style}
            label={props.label}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryRenderTargetColorData(props.id, props.index)
                        .then(event => {
                            const format = match(
                                event.data.value_type,
                                'Color',
                                ImageBufferFormat.Color,
                                'FloatColor',
                                ImageBufferFormat.FloatColor,
                                MatchAny,
                                ImageBufferFormat.Unsupported,
                            );
                            setPreview({
                                width: event.data.width,
                                height: event.data.height,
                                format,
                                bytes: event.binary.subarray(
                                    event.data.bytes_range.start,
                                    event.data.bytes_range.end,
                                ),
                            });
                        });
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.RenderTargetPreview = RenderTargetPreview;

function RenderTarget(props) {
    const [open, setOpen] = useState(false);
    const [info, setInfo] = useState(null);

    const stateChildren = open && !!info
        ? (<ObjectField object={info} open contentonly />)
        : null;

    const buffersPreviewChildren = open && !!info
        ? info.buffers.colors.map((item, index) => (
            <RenderTargetPreview
                key={index}
                id={props.id}
                label={`Color: ${item.id}`}
                index={index}
            />
        ))
        : null;

    return (
        <Section
            style={props.style}
            label={props.id}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryRenderTarget(props.id)
                        .then(event => setInfo(event.data.info));
                }
                setOpen(checked);
            }}
        >
            <Filter
                label="Use in filters"
                query={props.id}
                type={FilterType.RenderTarget}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            {stateChildren}
            {buffersPreviewChildren}
        </Section>
    );
}
widgets.RenderTarget = RenderTarget;

function RenderTargetsTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info
            .filter(id => !props.filters.enabled
                || props.filters.renderTargets.some(item => doesMatchPattern(id, item)))
            .map((item, index) => (
                <RenderTarget
                    key={index}
                    id={item}
                    filters={props.filters}
                    dispatchFilters={props.dispatchFilters}
                />
            ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listRenderTargets().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.RenderTargetsTab = RenderTargetsTab;

function readVertexAttribute(view, offset, attribute) {
    const byteOffset = offset + attribute[1];
    const styles = {
        cell: {
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            padding: 2,
            border: '1px dotted darkred',
        },
    };

    return match(
        attribute[0].value_type,
        'Scalar',
        () => (<span style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</span>),
        'Vec2F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Vec3F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 8, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Vec4F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 12, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat2F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 8, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 12, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat3F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 12, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 24, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 16, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 28, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 20, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 32, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat4F',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 16, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 32, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 48, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 20, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 36, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 52, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 24, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 40, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 56, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 12, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 28, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 44, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getFloat32(byteOffset + 60, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Integer',
        () => (<span style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</span>),
        'Vec2I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Vec3I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 8, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Vec4I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 12, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat2I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 8, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 12, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat3I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 12, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 24, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 16, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 28, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 20, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 32, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        'Mat4I',
        () => (
            <table>
                <tbody>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 16, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 32, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 48, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 4, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 20, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 36, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 52, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 8, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 24, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 40, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 56, isLittleEndian)}</td>
                    </tr>
                    <tr>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 12, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 28, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 44, isLittleEndian)}</td>
                        <td style={styles.cell}>{view.getInt32(byteOffset + 60, isLittleEndian)}</td>
                    </tr>
                </tbody>
            </table>
        ),
        MatchAny,
        () => (<pre>👽</pre>),
    );
}

function MeshVertexBufferPreview(props) {
    const { layout, bytes } = props;
    const styles = {
        accent: {
            color: 'white',
        },
        cell: {
            border: '1px solid orange',
            padding: 3,
        },
    };

    const attributeChildren = layout.attributes.map((attribute, index) => {
        const { id, count, value_type } = attribute[0];
        const name = count > 1 ? `${id}[${count}]` : id;
        return (
            <th key={index} style={{ ...styles.cell, ...styles.accent }}>
                {name}<br />({value_type})
            </th>
        );
    });

    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const count = bytes.byteLength / layout.bytesize;
    const vertexChildren = [];
    for (let index = 0; index < count; ++index) {
        const offset = layout.bytesize * index;
        const values = layout.attributes.map((attribute, index) => (
            <td key={index} style={styles.cell}>
                {readVertexAttribute(view, offset, attribute)}
            </td>
        ));
        vertexChildren.push((
            <tr key={index}>
                <td style={{ ...styles.cell, ...styles.accent }}>
                    {`${index}`}
                </td>
                {values}
            </tr>
        ));
    }

    return (
        <table style={props.style}>
            <thead>
                <tr>
                    <th style={styles.cell}>#</th>
                    {attributeChildren}
                </tr>
            </thead>
            <tbody>
                {vertexChildren}
            </tbody>
        </table>
    );
}
widgets.MeshVertexBufferPreview = MeshVertexBufferPreview;

function MeshVertexPreview(props) {
    const [open, setOpen] = useState(false);
    const [preview, setPreview] = useState(null);

    const children = open && preview ? (
        <MeshVertexBufferPreview
            layout={preview.layout}
            bytes={preview.bytes}
        />
    ) : null;

    return (
        <Section
            style={props.style}
            label={props.label}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryMeshVertexData(props.id, props.index)
                        .then(event => setPreview({
                            layout: event.data.layout,
                            bytes: event.binary.subarray(
                                event.data.bytes_range.start,
                                event.data.bytes_range.end,
                            ),
                        }));
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.MeshVertexPreview = MeshVertexPreview;

function MeshIndexBufferPreview(props) {
    const { bytes, drawMode } = props;
    const styles = {
        accent: {
            color: 'white',
        },
        cell: {
            border: '1px solid orange',
            padding: 3,
        },
    };

    const elements = match(
        drawMode,
        'Triangles',
        3,
        'Lines',
        2,
        MatchAny,
        1,
    );
    const view = new Uint32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 4);
    const count = view.length / elements;
    const children = [];
    for (let index = 0; index < count; ++index) {
        const offset = elements * index;
        const values = match(
            drawMode,
            'Triangles',
            () => [
                <td key="index" style={{ ...styles.cell, ...styles.accent }}>{`${index}`}</td>,
                <td key={0} style={styles.cell}>{`${view[offset]}`}</td>,
                <td key={1} style={styles.cell}>{`${view[offset + 1]}`}</td>,
                <td key={2} style={styles.cell}>{`${view[offset + 2]}`}</td>,
            ],
            'Lines',
            () => [
                <td key="index" style={{ ...styles.cell, ...styles.accent }}>{`${index}`}</td>,
                <td key={0} style={styles.cell}>{`${view[offset]}`}</td>,
                <td key={1} style={styles.cell}>{`${view[offset + 1]}`}</td>,
            ],
            MatchAny,
            () => [
                <td key="index" style={{ ...styles.cell, ...styles.accent }}>{`${index}`}</td>,
                <td key={0} style={styles.cell}>{`${view[offset]}`}</td>,
            ],
        );
        children.push((<tr key={index}>{values}</tr>));
    }

    return (
        <div style={props.style}>
            <Field label="Draw mode" inline>{drawMode}</Field>
            <table>
                <tbody>
                    {children}
                </tbody>
            </table>
        </div>
    );
}
widgets.MeshIndexBufferPreview = MeshIndexBufferPreview;

function MeshIndexPreview(props) {
    const [open, setOpen] = useState(false);
    const [preview, setPreview] = useState(null);

    const children = open && preview
        ? (<MeshIndexBufferPreview drawMode={preview.drawMode} bytes={preview.bytes} />)
        : null;

    return (
        <Section
            style={props.style}
            label={props.label}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryMeshIndexData(props.id)
                        .then(event => setPreview({
                            drawMode: event.data.draw_mode,
                            bytes: event.binary.subarray(
                                event.data.bytes_range.start,
                                event.data.bytes_range.end,
                            ),
                        }));
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.MeshIndexPreview = MeshIndexPreview;

function MeshBufferPreview(props) {
    const { layout, drawMode, vertexBytes, indexBytes } = props;
    const styles = {
        accent: {
            color: 'white',
        },
        cell: {
            border: '1px solid orange',
            padding: 3,
        },
    };

    const buffersChildren = layout.buffers.map((buffer, index) => (
        <td
            key={index}
            style={styles.cell}
            colSpan={buffer.attributes.length}>
            {`Buffer #${index}`}
        </td>
    ));
    const attributeChildren = layout.buffers.map(
        buffer => buffer.attributes.map((attribute, index) => {
            const { id, count, value_type } = attribute[0];
            const name = count > 1 ? `${id}[${count}]` : id;
            return (
                <th key={index} style={{ ...styles.cell, ...styles.accent }}>
                    {name}<br />({value_type})
                </th>
            );
        })
    ).flat();
    const elements = match(
        drawMode,
        'Triangles',
        3,
        'Lines',
        2,
        MatchAny,
        1,
    );
    const indexView = new Uint32Array(indexBytes.buffer, indexBytes.byteOffset, indexBytes.byteLength / 4);
    const vertexViews = vertexBytes.map(bytes => new DataView(
        bytes.buffer,
        bytes.byteOffset,
        bytes.byteLength,
    ));
    const indexCount = indexView.length / elements;
    const vertexChildren = [];
    for (let index = 0; index < indexCount; ++index) {
        for (let element = 0; element < elements; ++element) {
            const offset = indexView[elements * index + element];
            const group = element === 0 ? (
                <td style={styles.cell} rowSpan={elements}>{`${index}`}</td>
            ) : null;
            const values = layout.buffers.map(
                (buffer, bufferIndex) => buffer.attributes.map((attribute, index) => {
                    const vertexView = vertexViews[bufferIndex];
                    const byteOffset = offset * buffer.bytesize;
                    const value = byteOffset < vertexView.byteLength
                        ? readVertexAttribute(vertexView, byteOffset, attribute)
                        : (<pre>🗑️</pre>);
                    return (<td key={`${bufferIndex}-${index}`} style={styles.cell}>{value}</td>);
                })
            ).flat();
            vertexChildren.push((
                <tr key={`${index}-${element}`}>
                    {group}
                    <td style={{ ...styles.cell, ...styles.accent }}>
                        {`${offset}`}
                    </td>
                    {values}
                </tr>
            ));
        }
    }

    return (
        <div style={props.style}>
            <Field label="Draw mode" inline>{drawMode}</Field>
            <table>
                <thead>
                    <tr>
                        <th style={styles.cell} colSpan={2} rowSpan={2}>#</th>
                        {buffersChildren}
                    </tr>
                    <tr>
                        {attributeChildren}
                    </tr>
                </thead>
                <tbody>
                    {vertexChildren}
                </tbody>
            </table>
        </div>
    );
}
widgets.MeshBufferPreview = MeshBufferPreview;

function MeshPreview(props) {
    const [open, setOpen] = useState(false);
    const [preview, setPreview] = useState(null);

    const children = open && preview ? (
        <MeshBufferPreview
            layout={preview.layout}
            drawMode={preview.drawMode}
            vertexBytes={preview.vertexBytes}
            indexBytes={preview.indexBytes}
        />
    ) : null;

    return (
        <Section
            style={props.style}
            label={props.label}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryMeshData(props.id)
                        .then(event => setPreview({
                            layout: event.data.layout,
                            drawMode: event.data.draw_mode,
                            vertexBytes: event.data.vertex_bytes_ranges.map(item => event.binary.subarray(
                                item.start,
                                item.end,
                            )),
                            indexBytes: event.binary.subarray(
                                event.data.index_bytes_range.start,
                                event.data.index_bytes_range.end,
                            ),
                        }));
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.MeshPreview = MeshPreview;

function Mesh(props) {
    const [open, setOpen] = useState(false);
    const [info, setInfo] = useState(null);

    const infoChildren = open && !!info
        ? (<ObjectField object={info} open contentonly />)
        : null;

    const previewChildren = open && !!info
        ? (<MeshPreview id={props.id} label={`Mesh Preview`} />)
        : null;

    const vertexPreviewChildren = open && !!info
        ? info.vertex_data.map((_, index) => (
            <MeshVertexPreview
                key={index}
                id={props.id}
                index={index}
                label={`Vertex Buffer #${index} Preview`}
            />
        ))
        : null;

    const indexPreviewChildren = open && !!info
        ? (<MeshIndexPreview id={props.id} label={`Index Buffer Preview`} />)
        : null;

    return (
        <Section
            style={props.style}
            label={props.id}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryMesh(props.id)
                        .then(event => setInfo(event.data.info));
                }
                setOpen(checked);
            }}
        >
            <Filter
                label="Use in filters"
                query={props.id}
                type={FilterType.Mesh}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            {infoChildren}
            {previewChildren}
            {vertexPreviewChildren}
            {indexPreviewChildren}
        </Section>
    );
}
widgets.Mesh = Mesh;

function MeshesTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info
            .filter(id => !props.filters.enabled
                || props.filters.meshes.some(item => doesMatchPattern(id, item)))
            .map((item, index) => (
                <Mesh
                    key={index}
                    id={item}
                    filters={props.filters}
                    dispatchFilters={props.dispatchFilters}
                />
            ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listMeshes().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.MeshesTab = MeshesTab;

function ImagePreview(props) {
    const [open, setOpen] = useState(false);
    const [preview, setPreview] = useState(null);

    const children = open && preview ? (
        <ImageBufferPreview
            width={preview.width}
            height={preview.height}
            depth={preview.depth}
            format={preview.format}
            bytes={preview.bytes}
        />
    ) : null;

    return (
        <Section
            style={props.style}
            label={props.label}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryImageData(props.id)
                        .then(event => {
                            const format = match(
                                event.data.format,
                                'RGBA',
                                ImageBufferFormat.RGBA,
                                'RGB',
                                ImageBufferFormat.RGB,
                                'Luminance',
                                ImageBufferFormat.Luminance,
                                'Data',
                                ImageBufferFormat.Data,
                                MatchAny,
                                ImageBufferFormat.Unsupported,
                            );
                            setPreview({
                                width: event.data.width,
                                height: event.data.height,
                                depth: event.data.depth,
                                format,
                                bytes: event.binary.subarray(
                                    event.data.bytes_range.start,
                                    event.data.bytes_range.end,
                                ),
                            });
                        });
                }
                setOpen(checked);
            }}
        >
            {children}
        </Section>
    );
}
widgets.ImagePreview = ImagePreview;

function Image(props) {
    const [open, setOpen] = useState(false);
    const [info, setInfo] = useState(null);

    const children = open && !!info
        ? (<ObjectField key="state" object={info} open contentonly />)
        : null;

    return (
        <Section
            style={props.style}
            label={props.id}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryImage(props.id)
                        .then(event => setInfo(event.data.info));
                }
                setOpen(checked);
            }}
        >
            <Filter
                label="Use in filters"
                query={props.id}
                type={FilterType.Image}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            {children}
            <ImagePreview key="preview" id={props.id} label="Buffer preview" />
        </Section>
    );
}
widgets.Image = Image;

function ImagesTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info
            .filter(id => !props.filters.enabled
                || props.filters.images.some(item => doesMatchPattern(id, item)))
            .map((item, index) => (
                <Image
                    key={index}
                    id={item}
                    filters={props.filters}
                    dispatchFilters={props.dispatchFilters}
                />
            ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listImages().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.ImagesTab = ImagesTab;

function MaterialVersion(props) {
    const { signature, baked, index } = props;
    const [open, setOpen] = useState(false);
    const styles = {
        code: {
            padding: 6,
            borderRadius: 10,
            whiteSpace: 'pre-wrap',
            backgroundColor: 'rgba(0, 0, 0, 0.5)'
        },
    };

    return (
        <Section
            style={props.style}
            label={`#${index}`}
            open={open}
            onChange={checked => setOpen(checked)}
        >
            <Filter
                label="Use in filters"
                query={{
                    id: props.id,
                    signature,
                }}
                type={FilterType.Material}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            <fieldset>
                <legend>Signature</legend>
                <ObjectField label="Domain" object={signature.domain} open />
                <ObjectField label="Render target outputs" object={signature.render_target} open />
                <Field label="Mesh inputs" open>
                    {signature.mesh.map((item, index) => (
                        <Field key={index} label={item[0]} inline>{`${item[1]}`}</Field>
                    ))}
                </Field>
                <ObjectField label="Middlewares" object={signature.middlewares} open />
            </fieldset>
            <fieldset>
                <legend>Baked shader</legend>
                <ObjectField label="Uniforms" object={baked.uniforms} open />
                <ObjectField label="Samplers" object={baked.samplers} open />
                <AutoSection label="Vertex shader">
                    <pre style={styles.code}>{baked.vertex}</pre>
                </AutoSection>
                <AutoSection label="Fragment shader">
                    <pre style={styles.code}>{baked.fragment}</pre>
                </AutoSection>
            </fieldset>
        </Section>
    );
}
widgets.MaterialVersion = MaterialVersion;

function Material(props) {
    const [open, setOpen] = useState(false);
    const [info, setInfo] = useState(null);

    const children = open && !!info ? [
        <fieldset key="versions">
            <legend>Versions</legend>
            {
                info.versions
                    .filter(([signature]) => !props.filters.enabled
                        || props.filters.materials.some(
                            item => doesMatchPattern({ id: props.id, signature }, item, true)
                        )
                    )
                    .map(([signature, baked], index) => (
                        <MaterialVersion
                            key={index}
                            index={index}
                            id={props.id}
                            signature={signature}
                            baked={baked}
                            filters={props.filters}
                            dispatchFilters={props.dispatchFilters}
                        />
                    ))
            }
        </fieldset>,
        <fieldset key="default-values">
            <legend>Default values</legend>
            <ObjectField object={info.default_values} contentonly />
        </fieldset>,
    ] : null;

    return (
        <Section
            style={props.style}
            label={props.id}
            open={open}
            onChange={checked => {
                if (checked) {
                    bridge
                        .queryMaterial(props.id)
                        .then(event => setInfo(event.data.info));
                }
                setOpen(checked);
            }}
        >
            <Filter
                label="Use in filters"
                query={{
                    id: props.id,
                    signature: MatchAny,
                }}
                type={FilterType.Material}
                filters={props.filters}
                dispatchFilters={props.dispatchFilters}
            />
            {children}
        </Section>
    );
}
widgets.Material = Material;

function MaterialsTab(props) {
    const [info, setInfo] = useState(null);

    const children = !!info
        ? info
            .filter(id => !props.filters.enabled
                || props.filters.materials.some(
                    item => doesMatchPattern(item, { id, signature: MatchAny })
                )
            )
            .map((item, index) => (
                <Material
                    key={index}
                    id={item}
                    filters={props.filters}
                    dispatchFilters={props.dispatchFilters}
                />
            ))
        : null;

    return (
        <Tab
            style={props.style}
            onShow={() => bridge.listMaterials().then(event => setInfo(event.data))}
        >
            {children}
        </Tab>
    );
}
widgets.MaterialsTab = MaterialsTab;

function FiltersTab(props) {
    const { filters, dispatchFilters } = props;

    const renderTargetsChildren = filters.renderTargets.map((query, index) => (
        <Filter
            key={index}
            query={query}
            type={FilterType.RenderTarget}
            filters={filters}
            dispatchFilters={dispatchFilters}
        />
    ));
    const meshesChildren = filters.meshes.map((query, index) => (
        <Filter
            key={index}
            query={query}
            type={FilterType.Mesh}
            filters={filters}
            dispatchFilters={dispatchFilters}
        />
    ));
    const imagesChildren = filters.images.map((query, index) => (
        <Filter
            key={index}
            query={query}
            type={FilterType.Image}
            filters={filters}
            dispatchFilters={dispatchFilters}
        />
    ));
    const materialsChildren = filters.materials.map((query, index) => (
        <Filter
            key={index}
            query={query}
            type={FilterType.Material}
            filters={filters}
            dispatchFilters={dispatchFilters}
        />
    ));

    return (
        <Tab style={props.style}>
            <Toggle
                label="Enabled"
                on={filters.enabled}
                onChange={mode => dispatchFilters({ action: FilterAction.Enable, mode })}
            />
            <Conditional condition={
                renderTargetsChildren.length > 0
                || meshesChildren.length > 0
                || imagesChildren.length > 0
                || materialsChildren.length > 0
            }>
                <Button
                    label="Clear all"
                    onClick={() => dispatchFilters({ action: FilterAction.Clear })}
                />
            </Conditional>
            <Conditional condition={renderTargetsChildren.length > 0}>
                <AutoSection label="Render Targets" open>
                    <Button
                        label="Clear"
                        onClick={() => dispatchFilters({ action: FilterAction.Clear, type: FilterType.RenderTarget })}
                    />
                    {renderTargetsChildren}
                </AutoSection>
            </Conditional>
            <Conditional condition={meshesChildren.length > 0}>
                <AutoSection label="Meshes" open>
                    <Button
                        label="Clear"
                        onClick={() => dispatchFilters({ action: FilterAction.Clear, type: FilterType.Mesh })}
                    />
                    {meshesChildren}
                </AutoSection>
            </Conditional>
            <Conditional condition={imagesChildren.length > 0}>
                <AutoSection label="Images" open>
                    <Button
                        label="Clear"
                        onClick={() => dispatchFilters({ action: FilterAction.Clear, type: FilterType.Image })}
                    />
                    {imagesChildren}
                </AutoSection>
            </Conditional>
            <Conditional condition={materialsChildren.length > 0}>
                <AutoSection label="Materials" open>
                    <Button
                        label="Clear"
                        onClick={() => dispatchFilters({ action: FilterAction.Clear, type: FilterType.Material })}
                    />
                    {materialsChildren}
                </AutoSection>
            </Conditional>
        </Tab>
    );
}
widgets.FiltersTab = FiltersTab;

function SnapshotsTab(props) {
    const dirty = useDirty();

    const children = snapshots.map((snapshot, index) => (
        <AutoSection
            key={index}
            label={snapshot.timestamp}
            labelAccent={bridge.snapshot === snapshot}
        >
            <Conditional condition={bridge.snapshot !== snapshot}>
                <Button
                    label="Activate"
                    onClick={() => bridge.snapshot = snapshot}
                />
            </Conditional>
            <Button
                label="Delete"
                onClick={() => {
                    if (bridge.snapshot === snapshot) {
                        bridge.snapshot = null;
                    }
                    snapshots.splice(index, 1);
                    dirty();
                }}
            />
        </AutoSection>
    ));

    return (
        <Tab style={props.style}>
            <Button
                label="Take snapshot"
                onClick={() => bridge.takeSnapshot().then(() => dirty())}
            />
            <Conditional condition={!!bridge.snapshot}>
                <Button
                    label="Deactivate current snapshot"
                    onClick={() => bridge.snapshot = null}
                />
            </Conditional>
            <Conditional condition={snapshots.length > 0}>
                <Button
                    label="Delete all snapshots"
                    onClick={() => {
                        bridge.snapshot = null;
                        snapshots.splice(0, snapshots.length);
                    }}
                />
            </Conditional>
            <fieldset>
                <legend>Snapshots</legend>
                {children}
            </fieldset>
        </Tab>
    );
}
widgets.SnapshotsTab = SnapshotsTab;

function reduceFilters(state, action) {
    let { enabled, renderTargets, meshes, images, materials } = state;

    const target = match(
        action.type,
        FilterType.RenderTarget,
        renderTargets,
        FilterType.Mesh,
        meshes,
        FilterType.Image,
        images,
        FilterType.Material,
        materials,
        MatchAny,
        null,
    );

    match(
        action.action,
        FilterAction.Enable,
        () => enabled = !!action.mode,
        FilterAction.Add,
        () => {
            if (!!target && target.every(item => !doesMatchPattern(item, action.query))) {
                target.push(action.query);
            }
        },
        FilterAction.BulkAdd,
        () => {
            if (!!action.renderTargets) {
                for (const query of action.renderTargets) {
                    if (!!renderTargets && renderTargets.every(item => !doesMatchPattern(item, query))) {
                        renderTargets.push(query);
                    }
                }
            }
            if (!!action.meshes) {
                for (const query of action.meshes) {
                    if (!!meshes && meshes.every(item => !doesMatchPattern(item, query))) {
                        meshes.push(query);
                    }
                }
            }
            if (!!action.images) {
                for (const query of action.images) {
                    if (!!images && images.every(item => !doesMatchPattern(item, query))) {
                        images.push(query);
                    }
                }
            }
            if (!!action.materials) {
                for (const query of action.materials) {
                    if (!!materials && materials.every(item => !doesMatchPattern(item, query))) {
                        materials.push(query);
                    }
                }
            }
        },
        FilterAction.Remove,
        () => {
            if (!!target) {
                const indices = target
                    .map((item, index) => doesMatchPattern(item, action.query) ? index : -1)
                    .filter(index => index >= 0)
                    .reverse();
                for (const index of indices) {
                    target.splice(index, 1);
                }
            }
        },
        FilterAction.Clear,
        () => {
            if (!!target) {
                target.splice(0, target.length);
            } else {
                renderTargets.splice(0, renderTargets.length);
                meshes.splice(0, meshes.length);
                images.splice(0, images.length);
                materials.splice(0, materials.length);
            }
        },
    );

    return { enabled, renderTargets, meshes, images, materials };
}

function countFilters(filters) {
    return filters.renderTargets.length
        + filters.meshes.length
        + filters.images.length
        + filters.materials.length;
}

function App(props) {
    const [light, setLight] = useState(true);
    const [active, setActive] = useState(false);
    const [activeTab, setActiveTab] = useState('Stages');
    const [filters, dispatchFilters] = useReducer(reduceFilters, {
        enabled: false,
        renderTargets: [],
        meshes: [],
        images: [],
        materials: [],
    });
    const dirty = useDirty();

    useOpen(() => bridge.checkPulse().then(() => setActive(true)));
    useBridge('TakeSnapshot', event => {
        const { timestamp, data, binary } = event;
        snapshots.push({ timestamp, data, binary });
    });
    useBridge('SnapshotChanged', () => dirty());
    useOpen(() => {
        if (!!OxygengineHARD.startWithFiltersFromPipelines) {
            dispatchFilters({ action: FilterAction.Enable, mode: true });
            bridge.listPipelines().then(event => {
                for (const id of event.data) {
                    bridge
                        .queryPipelineResources(id)
                        .then(event => {
                            dispatchFilters({
                                action: FilterAction.BulkAdd,
                                renderTargets: event.data.render_targets,
                                meshes: event.data.meshes,
                                images: event.data.images,
                                materials: event.data.materials.map(([id, signature]) => {
                                    return { id, signature };
                                }),
                            });
                        });
                }
            });
        }
    });

    const styles = {
        container: {
            backgroundColor: light ? 'rgba(0, 0, 0, 0.25)' : 'rgba(0, 0, 0, 0.75)',
            width: '100%',
            minHeight: '100%',
            height: 'max-content',
            color: 'orange',
            fontFamily: 'monospace',
            fontWeight: 'bold',
            textShadow: '-1px -1px black, 0px -1px black, 1px -1px black, -1px 0px black, 1px 0px black, -1px 1px black, 0px 1px black, 1px 1px black',
            backdropFilter: 'blur(2px)',
        },
    };

    const content = active ?
        (
            <Tabs activeTab={activeTab} onChange={id => setActiveTab(id)}>
                <StagesTab key="Stages" />
                <PipelinesTab
                    key="Pipelines"
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <RenderTargetsTab
                    key="Render Targets"
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <MeshesTab
                    key="Meshes"
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <ImagesTab
                    key="Images"
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <MaterialsTab
                    key="Materials"
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <FiltersTab
                    key="Filters"
                    tabLabel={`Filters (${countFilters(filters)}): ${filters.enabled ? 'On' : 'Off'}`}
                    filters={filters}
                    dispatchFilters={dispatchFilters}
                />
                <SnapshotsTab
                    key="Snapshots"
                    tabLabel={!!bridge.snapshot ? `Snapshots (${bridge.snapshot.timestamp})` : 'Snapshots'}
                />
            </Tabs>
        ) :
        (<span>Waiting for game pulse...</span>);

    return (
        <div style={{ ...styles.container, ...props.style }}>
            <div>
                <span>Oxygengine HARD</span>
            </div>
            <Button
                label={light ? 'Dark background' : 'Light background'}
                onClick={() => setLight(!light)}
            />
            {content}
        </div>
    );
}
widgets.App = App;

ReactDOM.render(<App />, document.getElementById('oxygengine-hard-root'));
