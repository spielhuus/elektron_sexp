use lazy_static::lazy_static;
use ndarray::{arr2, s, Array, Array1, Array2};
use std::collections::HashMap;

use crate::error::Error;

use super::model::{Footprint, Graph, LibrarySymbol, Symbol};

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
    ]);
}

pub struct Shape {}

/// transform the coordinates to absolute values.
pub trait Transform<U, T> {
    fn transform(node: &U, pts: &T) -> T;
}
impl Transform<Symbol, Array2<f64>> for Shape {
    fn transform(symbol: &Symbol, pts: &Array2<f64>) -> Array2<f64> {
        let theta = -symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array2<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = &symbol.mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol.at + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Symbol, Array1<f64>> for Shape {
    fn transform(symbol: &Symbol, pts: &Array1<f64>) -> Array1<f64> {
        let theta = -symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = &symbol.mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol.at + verts;
        verts.mapv_into(|v| {
            let res = format!("{:.2}", v).parse::<f64>().unwrap();
            if res == -0.0 {
                0.0
            } else {
                res
            }
        })
    }
}
impl Transform<Footprint, Array2<f64>> for Shape {
    fn transform(footprint: &Footprint, pts: &Array2<f64>) -> Array2<f64> {
        let theta = /* TODO - */ footprint.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let verts: Array2<f64> = pts.dot(&rot);
        //verts = verts.dot(MIRROR.get(&symbol.mirror.join("")).unwrap());
        let verts = &footprint.at + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Footprint, Array1<f64>> for Shape {
    fn transform(symbol: &Footprint, pts: &Array1<f64>) -> Array1<f64> {
        let theta = /* TODO - */ symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let verts: Array1<f64> = pts.dot(&rot);
        //verts = verts.dot(MIRROR.get(&symbol.mirror.join("")).unwrap());
        let verts = &symbol.at + verts;
        verts.mapv_into(|v| {
            let res = format!("{:.2}", v).parse::<f64>().unwrap();
            if res == -0.0 {
                0.0
            } else {
                res
            }
        })
    }
}
/// transform the coordinates to absolute values.
pub trait Bounds<T> {
    fn bounds(&self, libs: &LibrarySymbol) -> Result<T, Error>;
}
impl Bounds<Array2<f64>> for Symbol {
    fn bounds(&self, libs: &LibrarySymbol) -> Result<Array2<f64>, Error> {
        let mut boundery: Array2<f64> = Array2::default((0, 2));
        let mut array = Vec::new();
        let mut rows: usize = 0;
        for symbol in &libs.symbols {
            if self.unit == symbol.unit || symbol.unit == 0 {
                for element in &symbol.graph {
                    match element {
                        Graph::Polyline(polyline) => {
                            for row in polyline.pts.rows() {
                                let x = row[0];
                                let y = row[1];
                                array.extend_from_slice(&[x, y]);
                                rows += 1;
                            }
                        }
                        Graph::Rectangle(rectangle) => {
                            array.extend_from_slice(&[rectangle.start[0], rectangle.start[1]]);
                            array.extend_from_slice(&[rectangle.end[0], rectangle.end[1]]);
                            rows += 2;
                        }
                        _ => {} //TODO: implement
                    }
                }
                for pin in &symbol.pin {
                    array.extend_from_slice(&[pin.at[0], pin.at[1]]);
                    rows += 1;
                }
            }
        }
        if rows > 0 {
            let array = Array::from_shape_vec((rows, 2), array).unwrap();
            let axis1 = array.slice(s![.., 0]);
            let axis2 = array.slice(s![.., 1]);
            boundery = arr2(&[
                [
                    *axis1
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        //.min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
                [
                    *axis1
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
            ]);
        }
        Ok(boundery)
    }
}

#[cfg(test)]
mod tests {
    use ndarray::arr2;

    use crate::Schema;
    use super::Bounds;

    #[test]
    fn shape_opamp_a() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("U1", 1).unwrap();
        let lib_symbol = doc.get_library("Amplifier_Operational:TL072").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-7.62, -5.08], [7.62, 5.08]]), size)
    }
    #[test]
    fn shape_opamp_c() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("U1", 3).unwrap();
        let lib_symbol = doc.get_library("Amplifier_Operational:TL072").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-2.54, -7.62], [-2.54, 7.62]]), size)
    }
    #[test]
    fn shape_r() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R1", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-1.016, -3.81], [1.016, 3.81]]), size)
    }
}
