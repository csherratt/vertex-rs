use std::collections::VecDeque;

use Polygon::{PolyQuad, PolyTri};
use {Polygon, Quad, Triangle};

/// provides a way to convert a polygon down to triangles
pub trait EmitTriangles {
    /// The content of each point in the face
    type Vertex;

    /// convert a polygon to one or more triangles, each triangle
    /// is returned by calling `emit`
    fn emit_triangles<F>(&self, F)
    where
        F: FnMut(Triangle<Self::Vertex>);
}

impl<T: Clone> EmitTriangles for Quad<T> {
    type Vertex = T;

    fn emit_triangles<F>(&self, mut emit: F)
    where
        F: FnMut(Triangle<T>),
    {
        let &Quad {
            ref x,
            ref y,
            ref z,
            ref w,
        } = self;
        emit(Triangle::new(x.clone(), y.clone(), z.clone()));
        emit(Triangle::new(z.clone(), w.clone(), x.clone()));
    }
}

impl<T: Clone> EmitTriangles for Triangle<T> {
    type Vertex = T;

    fn emit_triangles<F>(&self, mut emit: F)
    where
        F: FnMut(Triangle<T>),
    {
        emit(self.clone());
    }
}

impl<T: Clone> EmitTriangles for Polygon<T> {
    type Vertex = T;

    fn emit_triangles<F>(&self, emit: F)
    where
        F: FnMut(Triangle<T>),
    {
        match self {
            &PolyTri(ref t) => t.emit_triangles(emit),
            &PolyQuad(ref q) => q.emit_triangles(emit),
        }
    }
}

/// `Triangluate` is a easy to to convert any Polygon stream to
/// a stream of triangles. This is useful since Quads and other geometry
/// are not supported by modern graphics pipelines like OpenGL.
pub trait Triangulate<T, V> {
    /// convert a stream of Polygons to a stream of triangles
    fn triangulate(self) -> TriangulateIterator<T, V>;
}

impl<V, P: EmitTriangles<Vertex = V>, T: Iterator<Item = P>> Triangulate<T, V> for T {
    fn triangulate(self) -> TriangulateIterator<T, V> {
        TriangulateIterator::new(self)
    }
}

/// Used to iterator of polygons into a iterator of triangles
pub struct TriangulateIterator<SRC, V> {
    source: SRC,
    buffer: VecDeque<Triangle<V>>,
}

impl<V, U: EmitTriangles<Vertex = V>, SRC: Iterator<Item = U>> TriangulateIterator<SRC, V> {
    fn new(src: SRC) -> TriangulateIterator<SRC, V> {
        TriangulateIterator {
            source: src,
            buffer: VecDeque::new(),
        }
    }
}

impl<V, U: EmitTriangles<Vertex = V>, SRC: Iterator<Item = U>> Iterator
    for TriangulateIterator<SRC, V>
{
    type Item = Triangle<V>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (n, _) = self.source.size_hint();
        (n, None)
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.buffer.pop_front() {
                Some(v) => return Some(v),
                None => (),
            }

            match self.source.next() {
                Some(p) => p.emit_triangles(|v| self.buffer.push_back(v)),
                None => return None,
            }
        }
    }
}
