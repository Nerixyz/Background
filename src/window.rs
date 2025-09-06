use skia_safe::{
    ColorType, Surface,
    gpu::{
        BackendRenderTarget, DirectContext, Protected, SurfaceOrigin,
        d3d::{BackendContext, ID3D12Device, IDXGIAdapter1, TextureResourceInfo},
        surfaces,
    },
};
use windows::{
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_12_0,
            Direct3D12::{D3D12_RESOURCE_STATE_COMMON, D3D12CreateDevice},
            Dxgi::{
                Common::{
                    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                    DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                },
                CreateDXGIFactory1, DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_NONE,
                DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIFactory4,
                IDXGISwapChain3,
            },
        },
    },
    core::Interface,
};
use winit::{
    dpi::LogicalSize,
    event_loop::ActiveEventLoop,
    platform::windows::WindowAttributesExtWindows,
    window::{Window, WindowAttributes},
};

use crate::{
    layout::LayoutCtx,
    paint::{PaintCtx, Pipeline},
};

const BUFFER_COUNT: usize = 2;

pub struct DxWindow {
    pub window: Window,

    swap_chain: IDXGISwapChain3,
    direct_context: DirectContext,
    surfaces: [(Surface, BackendRenderTarget); BUFFER_COUNT],
}

impl DxWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        initial_size: skia_safe::Size,
        visible: bool,
    ) -> anyhow::Result<Self> {
        let mut window_attributes = WindowAttributes::default().with_class_name("x-dx-window");
        window_attributes.decorations = false;
        window_attributes.inner_size = Some(winit::dpi::Size::new(LogicalSize::new(
            initial_size.width,
            initial_size.height,
        )));
        window_attributes.title = "Background".into();
        window_attributes.visible = visible;

        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        let hwnd = HWND(u64::from(window.id()) as *mut _);
        let (width, height) = window.inner_size().into();

        let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory1() }?;
        let (adapter, device) = get_hardware_adapter_and_device(&factory)?;
        let queue = unsafe { device.CreateCommandQueue(&Default::default()) }?;

        let backend_context = BackendContext {
            adapter,
            device,
            queue,
            memory_allocator: None,
            protected_context: Protected::No,
        };
        let mut direct_context = unsafe { DirectContext::new_d3d(&backend_context, None) }.unwrap();

        let swap_chain: IDXGISwapChain3 = unsafe {
            factory.CreateSwapChainForHwnd(
                &backend_context.queue,
                hwnd,
                &DXGI_SWAP_CHAIN_DESC1 {
                    Width: width,
                    Height: height,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: BUFFER_COUNT as _,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    ..Default::default()
                },
                None,
                None,
            )
        }?
        .cast()?;

        let surfaces: [_; BUFFER_COUNT] = std::array::from_fn(|i| {
            let resource = unsafe { swap_chain.GetBuffer(i as u32).unwrap() };

            let backend_render_target = BackendRenderTarget::new_d3d(
                window.inner_size().into(),
                &TextureResourceInfo {
                    resource,
                    alloc: None,
                    resource_state: D3D12_RESOURCE_STATE_COMMON,
                    format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    sample_count: 1,
                    level_count: 0,
                    sample_quality_pattern: DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                    protected: Protected::No,
                },
            );

            let surface = surfaces::wrap_backend_render_target(
                &mut direct_context,
                &backend_render_target,
                SurfaceOrigin::TopLeft,
                ColorType::RGBA8888,
                None,
                None,
            )
            .unwrap();

            (surface, backend_render_target)
        });

        Ok(Self {
            window,
            swap_chain,
            direct_context,
            surfaces,
        })
    }

    pub fn render(&mut self, pipl: &mut Pipeline, layout: &LayoutCtx) {
        let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() };
        let (surface, _) = &mut self.surfaces[index as usize];
        let canvas = surface.canvas();
        let mut paint_ctx = PaintCtx::new(layout);
        pipl.paint(canvas, &mut paint_ctx);

        self.direct_context.flush_and_submit_surface(surface, None);

        unsafe { self.swap_chain.Present(1, DXGI_PRESENT::default()) }.unwrap();
    }
}

fn get_hardware_adapter_and_device(
    factory: &IDXGIFactory4,
) -> windows::core::Result<(IDXGIAdapter1, ID3D12Device)> {
    for i in 0.. {
        let adapter = unsafe { factory.EnumAdapters1(i) }?;

        let adapter_desc = unsafe { adapter.GetDesc1() }?;

        if (DXGI_ADAPTER_FLAG(adapter_desc.Flags as _) & DXGI_ADAPTER_FLAG_SOFTWARE)
            != DXGI_ADAPTER_FLAG_NONE
        {
            continue; // Don't select the Basic Render Driver adapter.
        }

        let mut device = None;
        if unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device) }.is_ok() {
            return Ok((adapter, device.unwrap()));
        }
    }
    unreachable!()
}
