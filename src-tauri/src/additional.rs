use tauri::{async_runtime::Mutex, LogicalPosition, LogicalSize, Runtime};

/*
```js
        let wm = new WindowManager("cider_main")
        const factor = await wm.scaleFactor();
        const size = await wm.outerSize();
        const logical = size.toLogical(factor);
        window.tempSize = logical;
        await wm.setSize(new LogicalSize(296, 296));
```
 */

lazy_static::lazy_static! {
    static ref OLD_SIZE: Mutex<(f32, f32)> = Mutex::new((600.0, 500.0));
    static ref OLD_POS: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));
}

#[tauri::command]
pub async fn set_miniplayer_mode<R: Runtime>(
    window: tauri::Window<R>,
    toggled: bool,
) -> Result<(), String> {
    let mut s_lock = OLD_SIZE.lock().await;
    let mut p_lock = OLD_POS.lock().await;

    if toggled {
        let factor = window.scale_factor().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let logical = size.to_logical::<f32>(factor);

        *s_lock = (logical.width, logical.height);
        drop(s_lock);

        let position = window.outer_position().map_err(|e| e.to_string())?;
        let logical_pos = position.to_logical::<f32>(factor);

        *p_lock = (logical_pos.x, logical_pos.y);
        drop(p_lock);

        window
            .set_title("Miniplayer - Cider")
            .map_err(|e| e.to_string())?;
        window.set_resizable(false).map_err(|e| e.to_string())?; // there is no benefit to changing the size, so i will disable it
        window
            .set_size(LogicalSize::new(296.0, 296.0))
            .map_err(|e| e.to_string())?;
        window.set_always_on_top(true).map_err(|e| e.to_string())?;

        Ok(())
    } else {
        let (w, h) = *s_lock;
        drop(s_lock);

        let (x, y) = *p_lock;
        drop(p_lock);

        window.set_title("Cider").map_err(|e| e.to_string())?;
        window
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
        window
            .set_size(LogicalSize::new(w, h))
            .map_err(|e| e.to_string())?;
        window.set_resizable(true).map_err(|e| e.to_string())?;
        window.set_always_on_top(false).map_err(|e| e.to_string())?;

        Ok(())
    }
}
