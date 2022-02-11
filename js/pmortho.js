// original: https://github.com/Falke-Design/PMOrtho
// this one's slightly modified for whole number snapping
// that guy's a legend
let Utils = {
    _enableKeyListener() {
        L.DomEvent.on(this.map.getContainer(), 'keydown', this._keyDownFunction, this);
        L.DomEvent.on(this.map.getContainer(), 'keyup', this._keyDownFunction, this);
        L.DomEvent.on(this.map.getContainer(), 'mouseover', this._keyDownFunction, this);
        this.map.boxZoom.disable();
    },
    _disableKeyListener() {
        L.DomEvent.off(this.map.getContainer(), 'keydown', this._keyDownFunction, this);
        L.DomEvent.off(this.map.getContainer(), 'keyup', this._keyDownFunction, this);
        L.DomEvent.off(this.map.getContainer(), 'mouseover', this._keyDownFunction, this);
        //Reset to default boxZoom
        if (this.map.pm.pmOrtho._defaultBox) {
            this.map.boxZoom.enable();
        }
    },
    _keyDownFunction(e) {
        if (e.type == "keyup") {
            this.map.pm.pmOrtho._shiftpressed = false;
            return;
        }
        if (this.map.pm.pmOrtho.options.customKey && this.map.pm.pmOrtho.options.customKey !== "shift") {
            let customKey = this.map.pm.pmOrtho.options.customKey;
            if (e.key == customKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else if (e.code == customKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else if (e.which == customKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else if (e.keyCode == customKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else if (customKey == "alt" && e.altKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else if ((customKey == "strg" || customKey == "ctrl") && e.ctrlKey) {
                this.map.pm.pmOrtho._shiftpressed = true;
            }
            else {
                this.map.pm.pmOrtho._shiftpressed = false;
            }
        }
        else {
            this.map.pm.pmOrtho._shiftpressed = e.shiftKey;
        }
    },
    _getPointofAngle(latlng_p1, latlng_p2, startAngle = 0) {
        let p1 = this.map.latLngToContainerPoint(latlng_p1);
        let p2 = this.map.latLngToContainerPoint(latlng_p2);
        let distance = this._getDistance(p1, p2);
        //Get the angle between the two points
        let pointAngle = this._getAngle(p1, p2);
        let snapAngle = this.options.snapAngle || 45;
        let angle = 0;
        let angles = [];
        let current = 0;
        let i = 0;
        while (i < (360 / snapAngle)) {
            current = ((i * snapAngle) + startAngle) % 360;
            angles.push(current);
            i++;
        }
        if (startAngle % 90 === 0) {
            angles.push(360);
        }
        angles.sort((a, b) => a - b);
        angle = angles.reduce(function (prev, curr) {
            return (Math.abs(curr - pointAngle) < Math.abs(prev - pointAngle) ? curr : prev);
        });
        let point_result2 = this._findDestinationPoint(p1, distance, angle);
        return this.map.containerPointToLatLng(point_result2);
    },
    _findDestinationPoint(point, distance, angle) {
        angle = (angle - 90) % 360;
        let x = Math.round(Math.cos(angle * Math.PI / 180) * distance + point.x);
        let y = Math.round(Math.sin(angle * Math.PI / 180) * distance + point.y);
        return { x: x, y: y };
    },
    _getDistance(p1, p2) {
        let x = p1.x - p2.x;
        let y = p1.y - p2.y;
        return Math.sqrt(x * x + y * y);
    },
    _getAngle(p1, p2) {
        let x = p2.x - p1.x;
        let y = p2.y - p1.y;
        let _angle = ((Math.atan2(y, x) * 180 / Math.PI) * (-1) - 90) * (-1);
        return (_angle < 0 ? _angle + 360 : _angle) % 360;
    },
    _getRectanglePoint(A, B) {
        let rect = L.rectangle([A, B]);
        let rectangleWidth = this.map.latLngToContainerPoint(A).x - this.map.latLngToContainerPoint(B).x;
        let rectangleHeight = this.map.latLngToContainerPoint(A).y - this.map.latLngToContainerPoint(B).y;
        let w = this.map.pm.pmOrtho._getDistance(this.map.latLngToContainerPoint(rect.getBounds().getNorthEast()), this.map.latLngToContainerPoint(rect.getBounds().getNorthWest()));
        let h = this.map.pm.pmOrtho._getDistance(this.map.latLngToContainerPoint(rect.getBounds().getNorthEast()), this.map.latLngToContainerPoint(rect.getBounds().getSouthEast()));
        let pt_A = this.map.latLngToContainerPoint(A);
        let pt_B = this.map.latLngToContainerPoint(B);
        let d;
        if (w > h) {
            const p = { x: pt_B.x, y: pt_A.y };
            const angle = rectangleHeight < 0 ? 180 : 0;
            d = this.map.pm.pmOrtho._findDestinationPoint(p, w, angle);
        }
        else {
            const p = { x: pt_A.x, y: pt_B.y };
            const angle = rectangleWidth < 0 ? 90 : -90;
            d = this.map.pm.pmOrtho._findDestinationPoint(p, h, angle);
        }
        return this.map.containerPointToLatLng(d);
    }
};
L.PMOrtho = L.Class.extend({
    includes: [Utils],
    options: {
        allowOrtho: true,
        customKey: null,
        snapAngle: 45,
        baseAngleOfLastSegment: false,
        showAngleTooltip: true,
        angleText: "Angle: "
    },
    cssadded: false,
    initialize(map, options) {
        this.map = map;
        L.setOptions(this, options);
        if (this.map && this.map.pm) {
            this.map.pm.pmOrtho = this;
        }
        this._overwriteFunctions();
    },
    setOptions(options) {
        L.setOptions(this, options);
    },
    _overwriteFunctions: function () {
        if (!L.PM.PMOrthoFunctionAdded) {
            L.PM.PMOrthoFunctionAdded = true;
            this._extendedEnable();
            this._extendedDisable();
            L.PM.Draw.Line.prototype._syncHintMarker = this._syncHintMarkerLine(this);
            L.PM.Draw.Rectangle.prototype._syncHintMarker = this._syncHintMarkerRectangle(this);
            L.PM.Draw.Rectangle.prototype._placeStartingMarkers = this._placeStartingMarkers(this);
            L.PM.Draw.Rectangle.prototype._finishShape = this._finishShape(this);
            L.PM.Draw.Marker.prototype._syncHintMarker = this._syncHintMarkerMarker(this);
            L.PM.Draw.Marker.prototype._createMarker = this._createMarker(this);
            L.PM.Edit.Line.prototype._addMarker = this._addMarker(this);
            L.PM.Edit.Line.prototype._onMarkerDragEnd = this._onMarkerDragEnd(this);
            L.PM.Edit.Rectangle.prototype._onMarkerDragEnd = this._onMarkerDragEndRectangle(this);
            L.PM.Edit.Rectangle.prototype._adjustRectangleForMarkerMove = this._adjustRectangleForMarkerMove(this);
            L.PM.Edit.prototype._onLayerDrag = this._onLayerDrag(this);
            L.PM.Edit.prototype._onRotateEnd = this._onRotateEnd(this);
            L.PM.Draw.Rectangle.prototype._finishShapeOrg = L.PM.Draw.Rectangle.prototype._finishShape;
            L.PM.Draw.Rectangle.prototype._finishShape = function (e) {
                e.latlng = this._cornerPoint || e.latlng;
                this._hintMarker._snapped = this._cornerPoint ? false : this._hintMarker._snapped;
                this._finishShapeOrg(e);
            };
            L.PM.Draw.prototype._addDrawnLayerPropOrg = L.PM.Draw.prototype._addDrawnLayerProp;
            L.PM.Draw.prototype._addDrawnLayerProp = function (layer) {
                this._addDrawnLayerPropOrg(layer);
                if (layer instanceof L.Rectangle) {
                    layer.pm._adjustRectangleForMarkerMoveOrg = layer.pm._adjustRectangleForMarkerMove;
                    layer.pm._adjustRectangleForMarkerMove = function (movedMarker) {
                        if (this._map.pm.pmOrtho._shiftpressed && this._map.pm.pmOrtho.options.allowOrtho) {
                            let newlatlng = this._map.pm.pmOrtho._getRectanglePoint(movedMarker._oppositeCornerLatLng, movedMarker.getLatLng());
                            movedMarker.setLatLng(newlatlng);
                        }
                        layer.pm._adjustRectangleForMarkerMoveOrg(movedMarker);
                        movedMarker.setLatLng(roundLatLng(movedMarker.getLatLng()));
                    };
                }
                else if (layer instanceof L.Polyline) {
                    layer.pm._onMarkerDragOrg = layer.pm._onMarkerDrag;
                    L.PM.Edit.Line._onMarkerDrag = function (e) {
                        const marker = e.target;
                        const { indexPath, index, parentPath } = this.findDeepMarkerIndex(this._markers, marker);
                        // only continue if this is NOT a middle marker
                        if (!indexPath) {
                            return;
                        }
                        // the dragged markers neighbors
                        const markerArr = indexPath.length > 1 ? _.get(this._markers, parentPath) : this._markers;
                        // find the indizes of next and previous markers
                        const prevMarkerIndex = (index + (markerArr.length - 1)) % markerArr.length;
                        const nextMarkerIndex = (index + (markerArr.length + 1)) % markerArr.length;
                        // get latlng of prev and next marker
                        const prevMarkerLatLng = markerArr[prevMarkerIndex].getLatLng();
                        const nextMarkerLatLng = markerArr[nextMarkerIndex].getLatLng();
                        if (this._map.pm.pmOrtho._shiftpressed && this._map.pm.pmOrtho.options.allowOrtho) {
                            let startAngle = 0;
                            if (this._map.pm.pmOrtho.options.baseAngleOfLastSegment && markerArr.length > 1) {
                                const prevPrevMarkerIndex = (index + (markerArr.length - 2)) % markerArr.length;
                                const prevPrevMarkerLatLng = markerArr[prevPrevMarkerIndex].getLatLng();
                                const lastPolygonPoint = this._map.latLngToContainerPoint(prevMarkerLatLng);
                                const secondLastPolygonPoint = this._map.latLngToContainerPoint(prevPrevMarkerLatLng);
                                startAngle = this._map.pm.pmOrtho._getAngle(secondLastPolygonPoint, lastPolygonPoint) + 90;
                                startAngle = startAngle > 180 ? startAngle - 180 : startAngle;
                            }
                            let newlatlng = this._map.pm.pmOrtho._getPointofAngle(prevMarkerLatLng, marker.getLatLng(), startAngle);
                            e.target._latlng = roundLatLng(newlatlng);
                            e.target.update();
                        }
                        if (markerArr.length > 1) {
                            if (!this._map.pm.pmOrtho._angleLine) {
                                this._map.pm.pmOrtho._angleLine = L.polyline([], { smoothFactor: 0 }).addTo(this._map);
                            }
                            const centerPoint = this._map.latLngToContainerPoint(marker.getLatLng());
                            const lastPolygonPoint = this._map.latLngToContainerPoint(prevMarkerLatLng);
                            const nextPolygonPoint = this._map.latLngToContainerPoint(nextMarkerLatLng);
                            let angle = this._map.pm.pmOrtho._getAngle(centerPoint, nextPolygonPoint);
                            angle = this._map.pm.pmOrtho._formatAngle(angle - this._map.pm.pmOrtho._getAngle(centerPoint, lastPolygonPoint));
                            let showTooltip = true;
                            if (this._layer instanceof L.Polygon) {
                                showTooltip = true;
                            }
                            else if (this._layer instanceof L.Polyline) {
                                if (prevMarkerIndex < (index + 1) && nextMarkerIndex !== 0) {
                                    showTooltip = true;
                                }
                                else {
                                    showTooltip = false;
                                }
                            }
                            if (this._map.pm.pmOrtho.options.showAngleTooltip && showTooltip) {
                                const coords = this._map.pm.pmOrtho._addAngleLine(prevMarkerLatLng, marker.getLatLng(), nextMarkerLatLng).getLatLngs();
                                if (coords.length === 0) {
                                    this._map.pm.pmOrtho._angleLine.remove();
                                    this._map.pm.pmOrtho._angleLine = null;
                                }
                                else {
                                    this._map.pm.pmOrtho._angleLine.setLatLngs(coords);
                                }
                                if (!this._map.pm.pmOrtho.tooltip) {
                                    this._map.pm.pmOrtho.tooltip = L.tooltip({
                                        permanent: true,
                                        offset: L.point(0, 10),
                                        direction: 'bottom',
                                        opacity: 0.8,
                                    }).setContent(this._map.pm.pmOrtho.options.angleText + angle).setLatLng(marker.getLatLng()).addTo(this._map);
                                }
                                else {
                                    this._map.pm.pmOrtho.tooltip.setLatLng(marker.getLatLng()).setContent(this._map.pm.pmOrtho.options.angleText + angle);
                                }
                            }
                            else {
                                this._map.pm.pmOrtho._angleLine.remove();
                                this._map.pm.pmOrtho._angleLine = null;
                                if (this._map.pm.pmOrtho.tooltip) {
                                    this._map.pm.pmOrtho.tooltip.remove();
                                    this._map.pm.pmOrtho.tooltip = null;
                                }
                            }
                        }
                        layer.pm._onMarkerDragOrg(e);
                    };
                    layer.pm._onMarkerDragEndOrg = layer.pm._onMarkerDragEnd;
                    layer.pm._onMarkerDragEnd = function (e) {
                        if (this._map.pm.pmOrtho._angleLine) {
                            this._map.pm.pmOrtho._angleLine.removeFrom(this._map.pm.pmOrtho.map);
                            this._map.pm.pmOrtho._angleLine = null;
                        }
                        if (this._map.pm.pmOrtho.tooltip) {
                            this._map.pm.pmOrtho.tooltip.removeFrom(this._map.pm.pmOrtho.map);
                            this._map.pm.pmOrtho.tooltip = null;
                        }
                        layer.pm._onMarkerDragEndOrg(e);
                    };
                }
            };
        }
        this.map.on('pm:create', (e) => {
            var map = e.target;
            if (map._angleLine) {
                map._angleLine.removeFrom(map);
                map._angleLine = null;
            }
            if (map.tooltip) {
                map.tooltip.removeFrom(map);
                map.tooltip = null;
            }
        });
        //Edit
        this.map.on('pm:globaleditmodetoggled', function (e) {
            if (e.enabled) {
                e.map.pm.pmOrtho._enableShiftListener();
            }
            else {
                e.map.pm.pmOrtho._disableShiftListener();
            }
        });
    },
    _extendedEnable() {
        L.PM.Draw.Line.prototype.enableOrg = L.PM.Draw.Line.prototype.enable;
        L.PM.Draw.Line.prototype.enable = function (options) {
            this.enableOrg(options);
            this._map.pm.pmOrtho._enableShiftListener();
            this._map.off('click', this._createVertex, this);
            this._map.on('click', this._map.pm.pmOrtho._createVertexNew, this);
        };
        L.PM.Draw.Rectangle.prototype.enableOrg = L.PM.Draw.Rectangle.prototype.enable;
        L.PM.Draw.Rectangle.prototype.enable = function (options) {
            this.enableOrg(options);
            this._map.pm.pmOrtho._enableShiftListener();
            if (this.options.cursorMarker) {
                L.DomUtil.addClass(this._hintMarker._icon, 'visible');
                // Add two more matching style markers, if cursor marker is rendered
                this._styleMarkers = [];
                for (let i = 0; i < 4; i += 1) {
                    const styleMarker = L.marker([0, 0], {
                        icon: L.divIcon({
                            className: 'marker-icon rect-style-marker',
                        }),
                        draggable: false,
                        zIndexOffset: 100,
                    });
                    styleMarker._pmTempLayer = true;
                    this._layerGroup.addLayer(styleMarker);
                    this._styleMarkers.push(styleMarker);
                }
            }
        };
    },
    _extendedDisable() {
        L.PM.Draw.Line.prototype.disableOrg = L.PM.Draw.Line.prototype.disable;
        L.PM.Draw.Line.prototype.disable = function () {
            this.disableOrg();
            this._map.pm.pmOrtho._disableShiftListener();
            this._map.off('click', this._map.pm.pmOrtho._createVertexNew, this);
            if (this._map.pm.pmOrtho._angleLine) {
                this._map.pm.pmOrtho._angleLine.remove();
                this._map.pm.pmOrtho._angleLine = null;
            }
            if (this._map.pm.pmOrtho.tooltip) {
                this._map.pm.pmOrtho.tooltip.remove();
                this._map.pm.pmOrtho.tooltip = null;
            }
        };
        L.PM.Draw.Rectangle.prototype.disableOrg = L.PM.Draw.Rectangle.prototype.disable;
        L.PM.Draw.Rectangle.prototype.disable = function () {
            this.disableOrg();
            this._map.pm.pmOrtho._disableShiftListener();
            if (this._map.pm.pmOrtho._angleLine) {
                this._map.pm.pmOrtho._angleLine.remove();
                this._map.pm.pmOrtho._angleLine = null;
            }
            if (this._map.pm.pmOrtho.tooltip) {
                this._map.pm.pmOrtho.tooltip.remove();
                this._map.pm.pmOrtho.tooltip = null;
            }
        };
        L.PM.Draw.Rectangle.include({ _syncRectangleSize: this._syncRectangleSize });
    },
    _enableShiftListener() {
        if (this.map.pm.pmOrtho.options.allowOrtho) {
            this.map.pm.pmOrtho._shiftpressed = false;
            this.map.pm.pmOrtho._defaultBox = this.map.boxZoom.enabled();
            this.map.pm.pmOrtho._enableKeyListener();
        }
    },
    _disableShiftListener() {
        if (this.map.pm.pmOrtho.options.allowOrtho) {
            this.map.pm.pmOrtho._disableKeyListener();
        }
    },
    _syncHintMarkerRectangle() {
        return function (e) {
            this._hintMarker.setLatLng(e.latlng);
            // if snapping is enabled, do it
            if (this.options.snappable) {
                const fakeDragEvent = e;
                fakeDragEvent.target = this._hintMarker;
                this._handleSnapping(fakeDragEvent);
            }
            this._hintMarker.setLatLng(roundLatLng(this._hintMarker.getLatLng()));
        };
    },
    _syncHintMarkerLine() {
        return function (e) {
            const polyPoints = this._layer.getLatLngs();
            if (polyPoints.length > 0 && this._map.pm.pmOrtho._shiftpressed && this._map.pm.pmOrtho.options.allowOrtho) {
                const lastPolygonLatLng = polyPoints[polyPoints.length - 1];
                let latlng_mouse = e.latlng;
                let startAngle = 0;
                if (this._map.pm.pmOrtho.options.baseAngleOfLastSegment && polyPoints.length > 1) {
                    const secondLastPolygonLatLng = polyPoints[polyPoints.length - 2];
                    const lastPolygonPoint = this._map.latLngToContainerPoint(lastPolygonLatLng);
                    const secondLastPolygonPoint = this._map.latLngToContainerPoint(secondLastPolygonLatLng);
                    startAngle = this._map.pm.pmOrtho._getAngle(secondLastPolygonPoint, lastPolygonPoint);
                }
                let pt = this._map.pm.pmOrtho._getPointofAngle(lastPolygonLatLng, latlng_mouse, startAngle);
                this._hintMarker.setLatLng(pt);
                e.latlng = pt; //Because of intersection
            }
            else {
                // move the cursor marker
                this._hintMarker.setLatLng(e.latlng);
            }
            // if snapping is enabled, do it
            if (this.options.snappable) {
                const fakeDragEvent = e;
                fakeDragEvent.target = this._hintMarker;
                this._handleSnapping(fakeDragEvent);
            }
            // if self-intersection is forbidden, handle it
            if (!this.options.allowSelfIntersection) {
                this._handleSelfIntersection(true, this._hintMarker.getLatLng());
            }
            this._hintMarker.setLatLng(roundLatLng(this._hintMarker.getLatLng())); // modification, nearest block
            if (polyPoints.length > 1) {
                if (!this._map.pm.pmOrtho._angleLine) {
                    this._map.pm.pmOrtho._angleLine = L.polyline([], { smoothFactor: 0 }).addTo(this._map);
                    this._map.pm.pmOrtho._angleLine.setStyle(this._layer.options);
                }
                let startAngle = 0;
                const secondLastPolygonLatLng = polyPoints[polyPoints.length - 2];
                const lastPolygonLatLng = polyPoints[polyPoints.length - 1];
                const lastPolygonPoint = this._map.latLngToContainerPoint(lastPolygonLatLng);
                if (this._map.pm.pmOrtho.options.baseAngleOfLastSegment && polyPoints.length > 1) {
                    const secondLastPolygonPoint = this._map.latLngToContainerPoint(secondLastPolygonLatLng);
                    startAngle = this._map.pm.pmOrtho._getAngle(secondLastPolygonPoint, lastPolygonPoint);
                }
                let latlng_mouse = this._hintMarker.getLatLng(); // because of snapping the hintmarker change position
                const mousePoint = this._map.latLngToContainerPoint(latlng_mouse);
                let angle = this._map.pm.pmOrtho._formatAngle(this._map.pm.pmOrtho._getAngle(mousePoint, lastPolygonPoint) - startAngle);
                if (this._map.pm.pmOrtho.options.showAngleTooltip) {
                    const coords = this._map.pm.pmOrtho._addAngleLine(secondLastPolygonLatLng, lastPolygonLatLng, latlng_mouse).getLatLngs();
                    if (coords.length === 0) {
                        this._map.pm.pmOrtho._angleLine.remove();
                        this._map.pm.pmOrtho._angleLine = null;
                    }
                    else {
                        this._map.pm.pmOrtho._angleLine.setLatLngs(coords);
                    }
                    if (!this._map.pm.pmOrtho.tooltip) {
                        this._map.pm.pmOrtho.tooltip = L.tooltip({
                            permanent: true,
                            offset: L.point(0, 10),
                            direction: 'bottom',
                            opacity: 0.8,
                        }).setContent(this._map.pm.pmOrtho.options.angleText + angle).setLatLng(lastPolygonLatLng).addTo(this._map);
                    }
                    else {
                        this._map.pm.pmOrtho.tooltip.setLatLng(lastPolygonLatLng).setContent(this._map.pm.pmOrtho.options.angleText + angle);
                    }
                }
                else {
                    this._map.pm.pmOrtho._angleLine.remove();
                    this._map.pm.pmOrtho._angleLine = null;
                    if (this._map.pm.pmOrtho.tooltip) {
                        this._map.pm.pmOrtho.tooltip.remove();
                        this._map.pm.pmOrtho.tooltip = null;
                    }
                }
            }
        };
    },
    _createVertexNew(e) {
        const polyPoints = this._layer.getLatLngs();
        if (polyPoints.length > 0 && this._map.pm.pmOrtho._shiftpressed && this._map.pm.pmOrtho.options.allowOrtho) {
            const lastPolygonLatLng = polyPoints[polyPoints.length - 1];
            let latlng_mouse = e.latlng;
            let startAngle = 0;
            if (this._map.pm.pmOrtho.options.baseAngleOfLastSegment && polyPoints.length > 1) {
                const secondLastPolygonLatLng = polyPoints[polyPoints.length - 2];
                const lastPolygonPoint = this._map.latLngToContainerPoint(lastPolygonLatLng);
                const secondLastPolygonPoint = this._map.latLngToContainerPoint(secondLastPolygonLatLng);
                startAngle = this._map.pm.pmOrtho._getAngle(secondLastPolygonPoint, lastPolygonPoint) + 90;
                startAngle = startAngle > 180 ? startAngle - 180 : startAngle;
            }
            if (this._map.pm.pmOrtho._angleLine) {
                this._map.pm.pmOrtho._angleLine.removeFrom(this._map);
                this._map.pm.pmOrtho._angleLine = null;
            }
            if (this._map.pm.pmOrtho.tooltip) {
                this._map.pm.pmOrtho.tooltip.removeFrom(this._map);
                this._map.pm.pmOrtho.tooltip = null;
            }
            let pt = this._map.pm.pmOrtho._getPointofAngle(lastPolygonLatLng, latlng_mouse, startAngle);
            e.latlng = pt; //Because of intersection
        }
        e.latlng = roundLatLng(e.latlng); // modification, nearest block
        this._createVertex(e);
    },
    _syncRectangleSize() {
        // Create a box using corners A & B (A = Starting Position, B = Current Mouse Position)
        const A = roundLatLng(this._startMarker.getLatLng());
        const B = roundLatLng(this._hintMarker.getLatLng());
        this._layer.setBounds([A, B]);
        if (this._map.pm.pmOrtho.options.allowOrtho && this._map.pm.pmOrtho._shiftpressed) {
            this._cornerPoint = roundLatLng(this._map.pm.pmOrtho._getRectanglePoint(A, B));
            this._layer.setBounds([A, this._cornerPoint]);
        }
        else {
            this._cornerPoint = null;
            this._layer.setBounds([
                roundLatLng(this._layer.getBounds()._southWest),
                roundLatLng(this._layer.getBounds()._northEast)
            ]);
        }
        // Add matching style markers, if cursor marker is shown
        if (this.options.cursorMarker && this._styleMarkers) {
            const corners = this._findCorners();
            // Reposition style markers
            corners.forEach((unmarkedCorner, index) => {
                this._styleMarkers[index].setLatLng(roundLatLng(unmarkedCorner));
            });
        }
    },
    _placeStartingMarkers(e) {
        return function (e) {
            // assign the coordinate of the click to the hintMarker, that's necessary for
            // mobile where the marker can't follow a cursor
            if (!this._hintMarker._snapped) {
                this._hintMarker.setLatLng(roundLatLng(e.latlng));
            }
            // get coordinate for new vertex by hintMarker (cursor marker)
            const latlng = this._hintMarker.getLatLng();
            // show and place start marker
            L.DomUtil.addClass(this._startMarker._icon, 'visible');
            this._startMarker.setLatLng(roundLatLng(latlng));
            // if we have the other two visibilty markers, show and place them now
            if (this.options.cursorMarker && this._styleMarkers) {
                this._styleMarkers.forEach((styleMarker) => {
                    L.DomUtil.addClass(styleMarker._icon, 'visible');
                    styleMarker.setLatLng(roundLatLng(latlng));
                });
            }
            this._map.off('click', this._placeStartingMarkers, this);
            this._map.on('click', this._finishShape, this);
            // change tooltip text
            this._hintMarker.setTooltipContent(L.PM.Utils.getTranslation('tooltips.finishRect'));
            this._setRectangleOrigin();
        };
    },
    _finishShape() {
        return function (e) {
            // assign the coordinate of the click to the hintMarker, that's necessary for
            // mobile where the marker can't follow a cursor
            if (!this._hintMarker._snapped) {
                this._hintMarker.setLatLng(e.latlng);
            }
            this._hintMarker.setLatLng(roundLatLng(this._hintMarker.getLatLng()));
            // get coordinate for new vertex by hintMarker (cursor marker)
            const B = this._hintMarker.getLatLng();
            // get already placed corner from the startmarker
            const A = this._startMarker.getLatLng();
            // If snap finish is required but the last marker wasn't snapped, do not finish the shape!
            if (this.options.requireSnapToFinish &&
                !this._hintMarker._snapped &&
                !this._isFirstLayer()) {
                return;
            }
            // create the final rectangle layer, based on opposite corners A & B
            const rectangleLayer = L.rectangle([A, B], this.options.pathOptions);
            // rectangle can only initialized with bounds (not working with rotation) so we update the latlngs
            if (this.options.rectangleAngle) {
                const corners = L.PM.Utils._getRotatedRectangle(A, B, this.options.rectangleAngle || 0, this._map);
                rectangleLayer.setLatLngs(corners);
                if (rectangleLayer.pm) {
                    rectangleLayer.pm._setAngle(this.options.rectangleAngle || 0);
                }
            }
            this._setPane(rectangleLayer, 'layerPane');
            this._finishLayer(rectangleLayer);
            rectangleLayer.addTo(this._map.pm._getContainingLayer());
            // fire the pm:create event and pass shape and layer
            this._fireCreate(rectangleLayer);
            // disable drawing
            this.disable();
            if (this.options.continueDrawing) {
                this.enable();
            }
        };
    },
    _syncHintMarkerMarker() {
        return function (e) {
            // move the cursor marker
            this._hintMarker.setLatLng(e.latlng);
            // if snapping is enabled, do it
            if (this.options.snappable) {
                const fakeDragEvent = e;
                fakeDragEvent.target = this._hintMarker;
                this._handleSnapping(fakeDragEvent);
            }
            this._hintMarker.setLatLng(roundLatLng(this._hintMarker.getLatLng()));
        };
    },
    _createMarker() {
        return function (e) {
            if (!e.latlng) {
                return;
            }
            // If snap finish is required but the last marker wasn't snapped, do not finish the shape!
            if (this.options.requireSnapToFinish &&
                !this._hintMarker._snapped &&
                !this._isFirstLayer()) {
                return;
            }
            // assign the coordinate of the click to the hintMarker, that's necessary for
            // mobile where the marker can't follow a cursor
            if (!this._hintMarker._snapped) {
                this._hintMarker.setLatLng(e.latlng);
            }
            // get coordinate for new vertex by hintMarker (cursor marker)
            const latlng = roundLatLng(this._hintMarker.getLatLng());
            // create marker
            const marker = new L.Marker(latlng, this.options.markerStyle);
            this._setPane(marker, 'markerPane');
            this._finishLayer(marker);
            if (!marker.pm) {
                // if pm is not create we don't apply dragging to the marker (draggable is applied to the marker, when it is added to the map )
                marker.options.draggable = false;
            }
            // add marker to the map
            marker.addTo(this._map.pm._getContainingLayer());
            if (marker.pm && this.options.markerEditable) {
                // enable editing for the marker
                marker.pm.enable();
            }
            else if (marker.dragging) {
                marker.dragging.disable();
            }
            // fire the pm:create event and pass shape and marker
            this._fireCreate(marker);
            this._cleanupSnapping();
            if (!this.options.continueDrawing) {
                this.disable();
            }
        };
    },
    _addAngleLine(p1, center, p2) {
        let b1 = turf.bearing(this._toLngLat(center), this._toLngLat(p1));
        let b2 = turf.bearing(this._toLngLat(center), this._toLngLat(p2));
        b1 = b1 < 0 ? b1 + 360 : b1; // bearing is by default between -180 and 180
        b2 = b2 < 0 ? b2 + 360 : b2;
        let dis1 = p1.distanceTo(center);
        let dis2 = p2.distanceTo(center);
        // set the smallest distance as radius
        let radius = dis1;
        if (dis2 < radius) {
            radius = dis2;
        }
        if (dis1 / dis2 < 0.4) { // smooth increasing of the circle, because the difference between the to lines is greater then 40%
            radius = dis1 * (1 - (dis1 / dis2));
        }
        else if (dis2 / dis1 < 0.4) { // smooth increasing of the circle, because the difference between the to lines is greater then 40%
            radius = dis2 * (1 - (dis2 / dis1));
        }
        else { // minimum circle-radius
            radius = radius * 0.4;
        }
        if (radius === 0) {
            return L.polyline([]);
        }
        // crete the sector (circle part) with 360 latlng points
        let x = turf.sector(this._toLngLat(center), radius / 1000, b1, b2, { steps: 360 });
        let polygon = L.geoJson(x).getLayers()[0]; // get the polygon from the geojson
        let polyline = L.polyline(polygon.getLatLngs()); // we want to return a polyline no polygon
        return polyline;
    },
    _toLngLat(latlng) {
        return [latlng.lng, latlng.lat];
    },
    _formatAngle(_angle) {
        let angle = (_angle < 0 ? _angle + 360 : _angle) % 360;
        var angleold = angle;
        if (this._shiftpressed) {
            const val = (angle % 5);
            if (val < 2.5 && val > 1) {
                angle = parseInt(angle) - parseInt(Math.ceil(val));
            }
            else if (val > 2.5) {
                angle = parseInt(angle) + parseInt(Math.ceil(5 - val));
            }
            else {
                angle = parseInt(angle);
            }
        }
        return angle.toFixed(2);
    },
    _addMarker() {
        return function (newM, leftM, rightM) {
            // first, make this middlemarker a regular marker
            newM.off('movestart', this._onMiddleMarkerMoveStart, this);
            newM.off(this.options.addVertexOn, this._onMiddleMarkerClick, this);
            // now, create the polygon coordinate point for that marker
            // and push into marker array
            // and associate polygon coordinate with marker coordinate
            newM.setLatLng(roundLatLng(newM.getLatLng()));
            const latlng = newM.getLatLng();
            const coords = this._layer._latlngs;
            // remove linked markers
            delete newM.leftM;
            delete newM.rightM;
            // the index path to the marker inside the multidimensional marker array
            const { indexPath, index, parentPath } = L.PM.Utils.findDeepMarkerIndex(this._markers, leftM);
            // define the coordsRing that is edited
            const coordsRing = indexPath.length > 1 ? _.get(coords, parentPath) : coords;
            // define the markers array that is edited
            const markerArr = indexPath.length > 1 ? _.get(this._markers, parentPath) : this._markers;
            // add coordinate to coordinate array
            coordsRing.splice(index + 1, 0, latlng);
            // add marker to marker array
            markerArr.splice(index + 1, 0, newM);
            // set new latlngs to update polygon
            this._layer.setLatLngs(coords);
            // create the new middlemarkers
            if (this.options.hideMiddleMarkers !== true) {
                this._createMiddleMarker(leftM, newM);
                this._createMiddleMarker(newM, rightM);
            }
            // fire edit event
            this._fireEdit();
            this._layerEdited = true;
            this._fireVertexAdded(newM, L.PM.Utils.findDeepMarkerIndex(this._markers, newM).indexPath, latlng);
            if (this.options.snappable) {
                this._initSnappableMarkers();
            }
        };
    },
    _onMarkerDragEnd() {
        return function (e) {
            const marker = e.target;
            marker.setLatLng(roundLatLng(marker.getLatLng()));
            if (!this._vertexValidationDragEnd(marker)) {
                return;
            }
            const { indexPath } = L.PM.Utils.findDeepMarkerIndex(this._markers, marker);
            // if self intersection is not allowed but this edit caused a self intersection,
            // reset and cancel; do not fire events
            let intersection = this.hasSelfIntersection();
            if (intersection &&
                this.options.allowSelfIntersectionEdit &&
                this._markerAllowedToDrag) {
                intersection = false;
            }
            const intersectionReset = !this.options.allowSelfIntersection && intersection;
            this._fireMarkerDragEnd(e, indexPath, intersectionReset);
            if (intersectionReset) {
                // reset coordinates
                this._layer.setLatLngs(this._coordsBeforeEdit);
                this._coordsBeforeEdit = null;
                // re-enable markers for the new coords
                this._initMarkers();
                if (this.options.snappable) {
                    this._initSnappableMarkers();
                }
                // check for selfintersection again (mainly to reset the style)
                this._handleLayerStyle();
                this._fireLayerReset(e, indexPath);
                return;
            }
            if (!this.options.allowSelfIntersection &&
                this.options.allowSelfIntersectionEdit) {
                this._handleLayerStyle();
            }
            // fire edit event
            this._fireEdit();
            this._layerEdited = true;
        };
    },
    _onMarkerDragEndRectangle() {
        return function (e) {
            // dragged marker
            const draggedMarker = e.target;
            draggedMarker.setLatLng(roundLatLng(draggedMarker.getLatLng()));
            if (!this._vertexValidationDragEnd(draggedMarker)) {
                return;
            }
            // Clean-up data attributes
            this._cornerMarkers.forEach((m) => {
                delete m._oppositeCornerLatLng;
            });
            this._fireMarkerDragEnd(e);
            // fire edit event
            this._fireEdit();
            this._layerEdited = true;
        };
    },
    _adjustRectangleForMarkerMove() {
        return function (movedMarker) {
            // update moved marker coordinates
            movedMarker.setLatLng(roundLatLng(movedMarker.getLatLng()));
            L.extend(movedMarker._origLatLng, movedMarker._latlng);
            // update rectangle boundaries, based on moved marker's new LatLng and cached opposite corner's LatLng
            const corners = L.PM.Utils._getRotatedRectangle(movedMarker.getLatLng(), movedMarker._oppositeCornerLatLng, this._angle || 0, this._map);
            this._layer.setLatLngs(corners.map(c => roundLatLng(c)));
            // Reposition the markers at each corner
            this._adjustAllMarkers();
            // Redraw the shape (to update altered rectangle)
            this._layer.redraw();
        };
    },
    _onLayerDrag() {
        return function (e) {
            // latLng of mouse event
            const latlng = roundLatLng(e.latlng);
            // delta coords (how far was dragged)
            const deltaLatLng = roundLatLng({
                lat: latlng.lat - this._tempDragCoord.lat,
                lng: latlng.lng - this._tempDragCoord.lng,
            });
            // move the coordinates by the delta
            const moveCoords = (coords) => 
            // alter the coordinates
            coords.map((currentLatLng) => {
                if (Array.isArray(currentLatLng)) {
                    // do this recursively as coords might be nested
                    return moveCoords(currentLatLng);
                }
                // move the coord and return it
                return {
                    lat: currentLatLng.lat + deltaLatLng.lat,
                    lng: currentLatLng.lng + deltaLatLng.lng,
                };
            });
            if (this._layer instanceof L.Circle ||
                (this._layer instanceof L.CircleMarker && this._layer.options.editable)) {
                // create the new coordinates array
                const newCoords = moveCoords([this._layer.getLatLng()]);
                // set new coordinates and redraw
                this._layer.setLatLng(newCoords[0]);
            }
            else if (this._layer instanceof L.CircleMarker ||
                this._layer instanceof L.Marker) {
                let coordsRefernce = this._layer.getLatLng();
                if (this._layer._snapped) {
                    // if layer is snapped we use the original latlng for re-calculation, else the layer will not be "unsnappable" anymore
                    coordsRefernce = this._layer._orgLatLng;
                }
                // create the new coordinates array
                const newCoords = moveCoords([coordsRefernce]);
                // set new coordinates and redraw
                this._layer.setLatLng(newCoords[0]);
            }
            else if (this._layer instanceof L.ImageOverlay) {
                // create the new coordinates array
                const newCoords = moveCoords([
                    this._layer.getBounds().getNorthWest(),
                    this._layer.getBounds().getSouthEast(),
                ]);
                // set new coordinates and redraw
                this._layer.setBounds(newCoords);
            }
            else {
                // create the new coordinates array
                const newCoords = moveCoords(this._layer.getLatLngs());
                // set new coordinates and redraw
                this._layer.setLatLngs(newCoords);
            }
            // save current latlng for next delta calculation
            this._tempDragCoord = latlng;
            e.layer = this._layer;
            // fire pm:dragstart event
            this._fireDrag(e);
        };
    },
    _onRotateEnd() {
        return function () {
            function copyLatLngs(layer, latlngs = layer.getLatLngs()) {
                if (layer instanceof L.Polygon) {
                    return L.polygon(latlngs).getLatLngs();
                }
                return L.polyline(latlngs).getLatLngs();
            }
            const startAngle = this._startAngle;
            delete this._rotationOriginLatLng;
            delete this._rotationOriginPoint;
            delete this._rotationStartPoint;
            delete this._initialRotateLatLng;
            delete this._startAngle;
            if (this._rotationLayer instanceof L.Polygon)
                this._rotationLayer.setLatLngs(this._rotationLayer.getLatLngs().map(cs => cs.map(c => roundLatLng(c))));
            else
                this._rotationLayer.setLatLngs(this._rotationLayer.getLatLngs().map(c => roundLatLng(c)));
            console.log(this._markers);
            this._markers.forEach(markerSet => markerSet.forEach(marker => marker.setLatLng(roundLatLng(marker.getLatLng()))));
            const originLatLngs = copyLatLngs(this._rotationLayer, this._rotationLayer.pm._rotateOrgLatLng);
            // store the new latlngs
            this._rotationLayer.pm._rotateOrgLatLng = copyLatLngs(this._rotationLayer);
            this._fireRotationEnd(this._rotationLayer, startAngle, originLatLngs);
            this._fireRotationEnd(this._map, startAngle, originLatLngs);
            this._rotationLayer.pm._fireEdit(this._rotationLayer, 'Rotation');
            this._preventRenderingMarkers(false);
        };
    }
});
